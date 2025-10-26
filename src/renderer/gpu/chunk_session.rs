use crate::compute::geo::{Frustum, Plane};
use crate::compute::num::ceil_div;
use crate::renderer::gpu::chunk_session_resources::GpuChunkSessionResources;
use crate::renderer::gpu::chunk_session_shader_types::{
    GPUChunkMeshEntry, GPUPackedIndirectArgsAtomic, GPUVoxelChunkHeader,
};
use crate::renderer::gpu::chunk_session_types::{ChunkMeshState, MeshStateError};
use crate::renderer::gpu::{GPUDispatchIndirectArgsAtomic, GPUVoxelChunk, GPUVoxelFaceData};
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::{Renderer, resources};
use crate::world::types::{CHUNK_DIM, Chunk};
use glam::IVec3;
use slabmap::SlabMap;
use suballoc::SubAllocator;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages,
    ComputePass, ComputePipeline, RenderPass,
};

#[derive(Debug, Clone, Copy)]
pub struct GpuChunkSessionConfig {
    pub max_indirect_count: u32,
    pub max_workgroup_size_1d: u32,
    pub max_workgroup_size_2d: u32,
    pub max_chunks: usize,
    pub max_write_count: usize,
    pub max_face_count: usize,
}

struct GpuState {
    mesh_allocator: SubAllocator,
    chunk_cache: SlabMap<IVec3, ChunkMeshState>,

    chunk_buffer: VxBuffer,
    aabb_visible_buffer: VxBuffer,
    write_batch_buffer: VxBuffer,
    meshing_batch_buffer: VxBuffer,
    voxel_face_buffer: VxBuffer,
    packed_indirect_buffer: VxBuffer,

    mdi_args_pipeline: ComputePipeline,
    mdi_args_bind_group: BindGroup,

    write_pipeline: ComputePipeline,
    write_bind_group: BindGroup,

    meshing_pipeline: ComputePipeline,
    meshing_bind_group: BindGroup,

    render_bind_group: BindGroup,
    render_bind_group_layout: BindGroupLayout,
}

impl GpuState {
    fn new(
        renderer: &Renderer<'_>,
        camera_buffer: &VxBuffer,
        config: GpuChunkSessionConfig,
    ) -> Self {
        let chunk_buffer = renderer.device.create_vx_buffer::<GPUVoxelChunk>(
            "Chunk Manager Chunks Buffer",
            config.max_chunks,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let write_batch_buffer = renderer.device.create_vx_buffer::<GPUVoxelChunk>(
            "Chunk Manager Chunks Write Buffer",
            config.max_write_count,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let voxel_face_buffer = renderer.device.create_vx_buffer::<GPUVoxelFaceData>(
            "Chunk Manager Face Data Buffer",
            config.max_face_count,
            BufferUsages::VERTEX | BufferUsages::STORAGE,
        );

        let meshing_batch_buffer = renderer.device.create_vx_buffer::<GPUChunkMeshEntry>(
            "Chunk Manager Mesh Queue Buffer",
            config.max_write_count,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let aabb_visible_buffer = renderer.device.create_vx_buffer::<GPUChunkMeshEntry>(
            "Chunk Manager AABB Visible Buffer",
            config.max_chunks, // fixme overkill but cheap anyway?
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let packed_indirect_buffer = renderer
            .device
            .create_vx_buffer::<GPUPackedIndirectArgsAtomic>(
                "Chunk Manager Draw Count And Dispatch Indirect Buffer",
                1,
                BufferUsages::INDIRECT | BufferUsages::STORAGE | BufferUsages::COPY_DST,
            );

        let mdi_args_bgl = GpuChunkSessionResources::mdi_args_bgl(
            &renderer.device,
            renderer.indirect_buffer.buffer_size,
            packed_indirect_buffer.buffer_size,
            meshing_batch_buffer.buffer_size,
            chunk_buffer.buffer_size,
            aabb_visible_buffer.buffer_size,
            camera_buffer.buffer_size,
        );
        let mdi_args_pipeline =
            GpuChunkSessionResources::mdi_args_pipeline(&renderer.device, &[&mdi_args_bgl]);
        let mdi_args_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk MDI Args Bind Group"),
            layout: &mdi_args_bgl,
            entries: &resources::utils::bind_entries([
                renderer.indirect_buffer.as_entire_binding(),
                packed_indirect_buffer.as_entire_binding(),
                meshing_batch_buffer.as_entire_binding(),
                chunk_buffer.as_entire_binding(),
                aabb_visible_buffer.as_entire_binding(),
                BindingResource::TextureView(&renderer.depth.mip_texture_array_view),
                camera_buffer.as_entire_binding(),
            ]),
        });

        let write_bgl = GpuChunkSessionResources::chunk_write_bgl(
            &renderer.device,
            chunk_buffer.buffer_size,
            write_batch_buffer.buffer_size,
        );
        let write_pipeline =
            GpuChunkSessionResources::chunk_write_pipeline(&renderer.device, &[&write_bgl]);
        let write_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Write Bind Group"),
            layout: &write_bgl,
            entries: &resources::utils::bind_entries([
                write_batch_buffer.as_entire_binding(),
                chunk_buffer.as_entire_binding(),
            ]),
        });

        let meshing_bgl = GpuChunkSessionResources::chunk_meshing_bgl(
            &renderer.device,
            chunk_buffer.buffer_size,
            voxel_face_buffer.buffer_size,
            meshing_batch_buffer.buffer_size,
        );
        let meshing_pipeline =
            GpuChunkSessionResources::chunk_meshing_pipeline(&renderer.device, &[&meshing_bgl]);
        let meshing_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Meshing Bind Group"),
            layout: &meshing_bgl,
            entries: &resources::utils::bind_entries([
                chunk_buffer.as_entire_binding(),
                voxel_face_buffer.as_entire_binding(),
                meshing_batch_buffer.as_entire_binding(),
            ]),
        });

        let (render_bind_group_layout, render_bind_group) =
            GpuChunkSessionResources::chunk_render_bind_group(
                &renderer.device,
                camera_buffer,
                &voxel_face_buffer,
            );

        let mesh_allocator = SubAllocator::new(config.max_face_count as u32);
        let chunk_cache = SlabMap::with_capacity(config.max_chunks);

        Self {
            mesh_allocator,
            chunk_cache,

            chunk_buffer,
            write_batch_buffer,
            voxel_face_buffer,
            meshing_batch_buffer,
            aabb_visible_buffer,
            packed_indirect_buffer,

            mdi_args_pipeline,
            mdi_args_bind_group,
            write_pipeline,
            write_bind_group,
            meshing_pipeline,
            meshing_bind_group,
            render_bind_group,
            render_bind_group_layout,
        }
    }

    fn cache_chunk(&mut self, chunk: &Chunk) -> GPUVoxelChunk {
        let mesh_meta = chunk.mesh_meta.unwrap();
        let mesh_state = ChunkMeshState::new_unmeshed(mesh_meta);
        let chunk_index = self.chunk_cache.insert(chunk.position, mesh_state);
        let header = GPUVoxelChunkHeader::new(chunk_index as u32, chunk.position);
        GPUVoxelChunk::new(header, chunk.adjacent_blocks, chunk.blocks)
    }

    fn drop_chunk(&mut self, position: &IVec3) {
        let (_, mesh_state) = self.chunk_cache.remove(position).unwrap();
        if let ChunkMeshState::Meshed(mesh_entry) = mesh_state {
            self.mesh_allocator
                .deallocate(mesh_entry.face_alloc)
                .unwrap();
        }
    }

    fn is_chunk_cached(&self, position: &IVec3) -> bool {
        self.chunk_cache.contains_key(position)
    }

    fn prepare_chunk_mesh_entry(
        &mut self,
        chunk_position: &IVec3,
    ) -> Result<GPUChunkMeshEntry, MeshStateError> {
        match self.chunk_cache.get_mut(chunk_position) {
            Some((chunk_index, render_mesh_state)) => match render_mesh_state {
                ChunkMeshState::Meshed(mesh_entry) => Ok(*mesh_entry),
                ChunkMeshState::Unmeshed(unmeshed_entry) if unmeshed_entry.has_faces() => {
                    let allocation = self
                        .mesh_allocator
                        .allocate(unmeshed_entry.total_faces)
                        .map_err(|_| MeshStateError::FailedAllocation)?;
                    render_mesh_state.set_as_meshed(chunk_index as u32, allocation);
                    Ok(render_mesh_state.entry_with_meshing_flag())
                }
                _ => Err(MeshStateError::Empty),
            },
            None => Err(MeshStateError::Missing),
        }
    }
}

struct CpuState {
    write_batch: Vec<GPUVoxelChunk>,
    aabb_visible_batch: Vec<GPUChunkMeshEntry>,
}

impl CpuState {
    fn new(config: GpuChunkSessionConfig) -> Self {
        // todo decent approximation for max visible chunks (not 1024)
        Self {
            write_batch: Vec::with_capacity(config.max_chunks),
            aabb_visible_batch: Vec::with_capacity(1024),
        }
    }
}

pub struct GpuChunkSession {
    pub config: GpuChunkSessionConfig,
    gpu_state: GpuState,
    cpu_state: CpuState,
}

impl GpuChunkSession {
    pub fn new(
        renderer: &Renderer<'_>,
        camera_buffer: &VxBuffer,
        config: GpuChunkSessionConfig,
    ) -> Self {
        let gpu_state = GpuState::new(renderer, camera_buffer, config);
        let cpu_state = CpuState::new(config);
        Self {
            config,
            gpu_state,
            cpu_state,
        }
    }

    pub fn drop_chunk(&mut self, position: &IVec3) {
        self.gpu_state.drop_chunk(position);
    }

    pub fn prepare_chunk_writes<'a>(&mut self, chunks: impl Iterator<Item = &'a Chunk>) {
        self.cpu_state.write_batch.clear();
        for chunk in chunks.take(self.config.max_write_count) {
            if self.gpu_state.is_chunk_cached(&chunk.position) {
                self.gpu_state.drop_chunk(&chunk.position);
            }
            let gpu_chunk = self.gpu_state.cache_chunk(chunk);
            self.cpu_state.write_batch.push(gpu_chunk);
        }
    }

    pub fn prepare_chunk_visibility(
        &mut self,
        view_planes: &[Plane; 6],
        mut missing_chunk: impl FnMut(IVec3),
    ) {
        let mut frustum_aabb = Frustum::aabb(view_planes);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();

        self.cpu_state.aabb_visible_batch.clear();
        frustum_aabb.discrete_points(|ch_pos| {
            match self.gpu_state.prepare_chunk_mesh_entry(&ch_pos) {
                Ok(chunk_mesh_entry) => {
                    self.cpu_state.aabb_visible_batch.push(chunk_mesh_entry);
                }
                Err(MeshStateError::Missing) => missing_chunk(ch_pos),
                Err(_) => (),
            };
        });
    }

    pub fn compute_chunk_writes(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        if self.cpu_state.write_batch.is_empty() {
            return;
        }
        renderer.write_buffer(
            &self.gpu_state.write_batch_buffer,
            0,
            bytemuck::cast_slice(&self.cpu_state.write_batch),
        );
        compute_pass.set_pipeline(&self.gpu_state.write_pipeline);
        compute_pass.set_bind_group(0, &self.gpu_state.write_bind_group, &[]);
        let batch_size = self.cpu_state.write_batch.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_2d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);
    }

    pub fn compute_chunk_visibility_and_meshing(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        if self.cpu_state.aabb_visible_batch.is_empty() {
            return;
        }
        // reset indirect args
        let dispatch_indirect = GPUDispatchIndirectArgsAtomic::new(0, 1, 1);
        let packed_indirect_args = GPUPackedIndirectArgsAtomic::new(0u32, dispatch_indirect);
        renderer.write_buffer(
            &self.gpu_state.packed_indirect_buffer,
            0,
            packed_indirect_args.as_bytes(),
        );
        renderer.write_buffer(
            &self.gpu_state.aabb_visible_buffer,
            0,
            bytemuck::cast_slice(&self.cpu_state.aabb_visible_batch),
        );
        // update mdi args and meshing queue
        compute_pass.set_pipeline(&self.gpu_state.mdi_args_pipeline);
        compute_pass.set_bind_group(0, &self.gpu_state.mdi_args_bind_group, &[]);
        let batch_size = self.cpu_state.aabb_visible_batch.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_2d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);

        // handle meshing queue
        compute_pass.set_pipeline(&self.gpu_state.meshing_pipeline);
        compute_pass.set_bind_group(0, &self.gpu_state.meshing_bind_group, &[]);
        compute_pass.dispatch_workgroups_indirect(&self.gpu_state.packed_indirect_buffer, 4 * 4);
    }

    pub fn render_chunks(&mut self, renderer: &Renderer<'_>, render_pass: &mut RenderPass) {
        if self.cpu_state.aabb_visible_batch.is_empty() {
            return;
        }
        render_pass.set_bind_group(1, &self.gpu_state.render_bind_group, &[]);
        render_pass.multi_draw_indirect_count(
            &renderer.indirect_buffer,
            0,
            &self.gpu_state.packed_indirect_buffer,
            0,
            self.config.max_indirect_count,
        );
    }

    pub fn render_bgl(&self) -> &BindGroupLayout {
        &self.gpu_state.render_bind_group_layout
    }
}
