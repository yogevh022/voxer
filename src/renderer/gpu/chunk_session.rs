use crate::compute::geo::{AABB, Frustum, Plane};
use crate::compute::num::ceil_div;
use crate::compute::utils::free_ptr;
use crate::renderer::gpu::chunk_session_mesh_data::{TRANSPARENT_LAYER_BLOCKS, chunk_mesh_data};
use crate::renderer::gpu::chunk_session_resources::GpuChunkSessionResources;
use crate::renderer::gpu::chunk_session_shader_types::{
    GPUChunkMeshEntry, GPUPackedIndirectArgsAtomic, GPUVoxelChunkHeader,
};
use crate::renderer::gpu::chunk_session_types::ChunkMeshEntry;
use crate::renderer::gpu::vx_gpu_delta_vec::VxGpuSyncVec;
use crate::renderer::gpu::{
    GPUChunkMeshEntryWrite, GPUDispatchIndirectArgsAtomic, GPUVoxelChunk, GPUVoxelFaceData,
};
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::{Renderer, resources};
use crate::world::block::VoxelBlock;
use crate::world::chunk::VoxelChunk;
use crate::world::{CHUNK_DIM, VoxelChunkAdjBlocks};
use glam::IVec3;
use spatialmap::SpatialMap;
use suballoc::SubAllocator;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindingResource, BufferUsages, ComputePass,
    ComputePipeline, RenderPass,
};

#[derive(Debug, Clone, Copy)]
pub struct GpuChunkSessionConfig {
    pub max_indirect_count: u32,
    pub max_workgroup_size_1d: u32,
    pub max_workgroup_size_2d: u32,
    pub max_chunks: usize,
    pub max_write_count: usize,
    pub max_face_count: usize,
    pub chunk_render_distance: usize,
}

struct GPUResources {
    chunks_buffer: VxBuffer,
    view_buffer: VxBuffer,
    view_write_buffer: VxBuffer,
    chunks_write_buffer: VxBuffer,
    meshing_buffer: VxBuffer,
    voxel_face_buffer: VxBuffer,
    packed_indirect_buffer: VxBuffer,

    mdi_args_pipeline: ComputePipeline,
    mdi_args_bind_group: BindGroup,

    chunk_write_pipeline: ComputePipeline,
    chunk_write_bind_group: BindGroup,

    view_chunks_write_pipeline: ComputePipeline,
    view_chunks_write_bg: BindGroup,

    meshing_pipeline: ComputePipeline,
    meshing_bind_group: BindGroup,

    render_bind_group: BindGroup,
    render_bind_group_layout: BindGroupLayout,
}

impl GPUResources {
    fn new(
        renderer: &Renderer<'_>,
        camera_buffer: &VxBuffer,
        config: GpuChunkSessionConfig,
    ) -> Self {
        let chunks_buffer = renderer.device.create_vx_buffer::<GPUVoxelChunk>(
            "Chunk Manager Chunks Buffer",
            config.max_chunks,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let chunks_write_buffer = renderer.device.create_vx_buffer::<GPUVoxelChunk>(
            "Chunk Manager Chunks Write Buffer",
            config.max_write_count,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let voxel_face_buffer = renderer.device.create_vx_buffer::<GPUVoxelFaceData>(
            "Chunk Manager Face Data Buffer",
            config.max_face_count,
            BufferUsages::VERTEX | BufferUsages::STORAGE,
        );

        let meshing_buffer = renderer.device.create_vx_buffer::<GPUChunkMeshEntry>(
            "Chunk Manager Mesh Queue Buffer",
            config.max_write_count,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let view_buffer = renderer.device.create_vx_buffer::<GPUChunkMeshEntry>(
            "Chunk Manager View Candidates Buffer",
            config.max_chunks, // fixme overkill but cheap anyway?
            BufferUsages::STORAGE,
        );

        let view_write_buffer = renderer.device.create_vx_buffer::<GPUChunkMeshEntryWrite>(
            "Chunk Manager View Candidates Write Buffer",
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
            meshing_buffer.buffer_size,
            chunks_buffer.buffer_size,
            view_buffer.buffer_size,
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
                meshing_buffer.as_entire_binding(),
                chunks_buffer.as_entire_binding(),
                view_buffer.as_entire_binding(),
                BindingResource::TextureView(&renderer.depth.mip_texture_array_view),
                camera_buffer.as_entire_binding(),
            ]),
        });

        let chunk_write_bgl = GpuChunkSessionResources::write_bgl(
            &renderer.device,
            chunks_write_buffer.buffer_size,
            chunks_buffer.buffer_size,
        );
        let chunk_write_pipeline =
            GpuChunkSessionResources::chunk_write_pipeline(&renderer.device, &[&chunk_write_bgl]);
        let chunk_write_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Write Bind Group"),
            layout: &chunk_write_bgl,
            entries: &resources::utils::bind_entries([
                chunks_write_buffer.as_entire_binding(),
                chunks_buffer.as_entire_binding(),
            ]),
        });

        let view_chunks_write_bgl = GpuChunkSessionResources::write_bgl(
            &renderer.device,
            view_write_buffer.buffer_size,
            view_buffer.buffer_size,
        );
        let view_chunks_write_pipeline = GpuChunkSessionResources::view_candidate_write_pipeline(
            &renderer.device,
            &[&view_chunks_write_bgl],
        );
        let view_chunks_write_bg = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Visible Candidate Write Bind Group"),
            layout: &view_chunks_write_bgl,
            entries: &resources::utils::bind_entries([
                view_write_buffer.as_entire_binding(),
                view_buffer.as_entire_binding(),
            ]),
        });

        let meshing_bgl = GpuChunkSessionResources::chunk_meshing_bgl(
            &renderer.device,
            chunks_buffer.buffer_size,
            voxel_face_buffer.buffer_size,
            meshing_buffer.buffer_size,
        );
        let meshing_pipeline =
            GpuChunkSessionResources::chunk_meshing_pipeline(&renderer.device, &[&meshing_bgl]);
        let meshing_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Meshing Bind Group"),
            layout: &meshing_bgl,
            entries: &resources::utils::bind_entries([
                chunks_buffer.as_entire_binding(),
                voxel_face_buffer.as_entire_binding(),
                meshing_buffer.as_entire_binding(),
            ]),
        });

        let (render_bind_group_layout, render_bind_group) =
            GpuChunkSessionResources::chunk_render_bind_group(
                &renderer.device,
                camera_buffer,
                &voxel_face_buffer,
            );

        Self {
            chunks_buffer,
            view_buffer,
            view_write_buffer,
            chunks_write_buffer,
            meshing_buffer,
            voxel_face_buffer,
            packed_indirect_buffer,

            mdi_args_pipeline,
            mdi_args_bind_group,
            chunk_write_pipeline,
            chunk_write_bind_group,
            view_chunks_write_pipeline,
            view_chunks_write_bg,
            meshing_pipeline,
            meshing_bind_group,
            render_bind_group,
            render_bind_group_layout,
        }
    }
}
struct CPUResources {
    mesh_allocator: SubAllocator,

    chunks: SpatialMap<ChunkMeshEntry>,
    chunks_adj: SpatialMap<VoxelChunkAdjBlocks>,
    chunks_write: Vec<GPUVoxelChunk>,
    chunks_write_headers: Vec<GPUVoxelChunkHeader>,
    view_chunks: VxGpuSyncVec<GPUChunkMeshEntry>,

    view_box: AABB,
    view_delta_add_pos: Vec<IVec3>,
    view_delta_add: Vec<AABB>,
    view_delta_del: Vec<AABB>,
}

impl CPUResources {
    fn new(config: GpuChunkSessionConfig) -> Self {
        let max_view_count_est = config.max_chunks / 4; // very conservative
        Self {
            mesh_allocator: SubAllocator::new(config.max_face_count as u32),

            chunks: SpatialMap::with_capacity([config.chunk_render_distance as u32 * 2; 3]),
            chunks_adj: SpatialMap::with_capacity([config.chunk_render_distance as u32 * 2; 3]),
            chunks_write: Vec::with_capacity(config.max_chunks),
            chunks_write_headers: Vec::with_capacity(config.max_chunks),
            view_chunks: VxGpuSyncVec::new(config.max_chunks, max_view_count_est),

            view_box: AABB::zero(),
            view_delta_add_pos: Vec::with_capacity(max_view_count_est),
            view_delta_add: Vec::with_capacity(6),
            view_delta_del: Vec::with_capacity(6),
        }
    }

    fn insert_chunk(&mut self, chunk: &VoxelChunk) -> GPUVoxelChunk {
        let pos = chunk.position;
        let index = self.chunks.index(pos);

        // insert chunk and deallocate pre-existing mesh
        let blocks_as_adj = chunk.blocks_as_adj();
        self.chunks_adj.insert_index(index, pos, blocks_as_adj);
        if let Some(mut prev) = self.chunks.insert_index(index, pos, Default::default()) {
            self.deallocate_mesh(&mut prev.value);
        }

        // add view entry if on screen
        if self.view_box.contains_point(pos.as_vec3()) {
            self.view_delta_add_pos.push(pos);
        }

        let header = GPUVoxelChunkHeader::new(index as u32, pos);
        GPUVoxelChunk::new_uninit(header, chunk.blocks)
    }

    fn prepare_chunk_writes<'a>(&mut self, chunks: impl Iterator<Item = &'a VoxelChunk>) {
        self.chunks_write.clear();
        self.chunks_write_headers.clear();
        for chunk in chunks {
            let gpu_chunk = self.insert_chunk(chunk);
            self.chunks_write_headers.push(gpu_chunk.header);
            self.chunks_write.push(gpu_chunk);
        }

        for i in 0..self.chunks_write.len() {
            let header = unsafe { self.chunks_write_headers.get_unchecked(i) };
            let index = header.index();
            let adj_blocks = self.adj_blocks_of(header.position());
            let gpu_chunk = &mut self.chunks_write[i];

            unsafe {
                let blocks = std::mem::transmute(gpu_chunk.content);
                gpu_chunk.adj_content = std::mem::transmute(adj_blocks);
                let mesh_state = ChunkMeshEntry::new(chunk_mesh_data(&blocks, &adj_blocks), index);
                let prev = self.chunks.get_index_mut_unchecked(index as usize);
                prev.value = mesh_state;
            };
        }
    }

    fn deallocate_mesh(&mut self, mesh_entry: &mut ChunkMeshEntry) {
        if let ChunkMeshEntry::GPU(gpu_entry) = mesh_entry {
            let alloc = gpu_entry.face_alloc;
            self.mesh_allocator.deallocate(alloc).unwrap();
            self.view_chunks.remove(gpu_entry.index as usize);
            *mesh_entry = ChunkMeshEntry::CPU(gpu_entry.as_cpu());
        }
    }

    fn allocate_mesh(&mut self, mesh_entry: &mut ChunkMeshEntry) {
        if let ChunkMeshEntry::CPU(cpu_entry) = mesh_entry {
            if cpu_entry.face_count != 0 {
                let alloc = self.mesh_allocator.allocate(cpu_entry.face_count).unwrap();
                let gpu_entry = cpu_entry.as_gpu(cpu_entry.index, alloc);
                *mesh_entry = ChunkMeshEntry::GPU(gpu_entry);
                self.view_chunks.push(gpu_entry);
            }
        }
    }

    fn allocate_view_delta(&mut self) {
        let delta_del = free_ptr(&mut self.view_delta_del);
        for pos in delta_del.drain(..).flat_map(|db| db.discrete_points()) {
            if let Some(mesh_entry_cell) = self.chunks.get_exact_mut(pos) {
                let mesh_entry = free_ptr(mesh_entry_cell);
                self.deallocate_mesh(&mut mesh_entry.value);
            }
        }
        let delta_add = free_ptr(&mut self.view_delta_add);
        for pos in delta_add.drain(..).flat_map(|db| db.discrete_points()) {
            if let Some(mesh_entry_cell) = self.chunks.get_exact_mut(pos) {
                let mesh_entry = free_ptr(mesh_entry_cell);
                self.allocate_mesh(&mut mesh_entry.value);
            }
        }
        for pos in free_ptr(&mut self.view_delta_add_pos).drain(..) {
            if let Some(mesh_entry_cell) = self.chunks.get_exact_mut(pos) {
                let mesh_entry = free_ptr(mesh_entry_cell);
                self.allocate_mesh(&mut mesh_entry.value);
            };
        }
    }

    fn update_view_delta(&mut self, new_box: AABB) {
        self.view_delta_add.clear();
        self.view_delta_del.clear();
        new_box.diff_out(self.view_box, &mut self.view_delta_add);
        self.view_box.diff_out(new_box, &mut self.view_delta_del);
        self.view_box = new_box;
    }

    fn adj_or_transparent(&self, position: IVec3, index: usize) -> [[VoxelBlock; 16]; 16] {
        match self.chunks_adj.get_exact(position) {
            Some(adj) => adj.value[index],
            None => TRANSPARENT_LAYER_BLOCKS,
        }
    }

    fn adj_blocks_of(&self, position: IVec3) -> VoxelChunkAdjBlocks {
        [
            self.adj_or_transparent(position.with_x(position.x + 1), 0), // px
            self.adj_or_transparent(position.with_y(position.y + 1), 1), // py
            self.adj_or_transparent(position.with_z(position.z + 1), 2), // pz
            self.adj_or_transparent(position.with_x(position.x - 1), 3), // mx
            self.adj_or_transparent(position.with_y(position.y - 1), 4), // my
            self.adj_or_transparent(position.with_z(position.z - 1), 5), // mz
        ]
    }
}

pub struct GpuChunkSession {
    pub config: GpuChunkSessionConfig,
    cpu: CPUResources,
    gpu: GPUResources,
}

impl GpuChunkSession {
    pub fn new(
        renderer: &Renderer<'_>,
        camera_buffer: &VxBuffer,
        config: GpuChunkSessionConfig,
    ) -> Self {
        Self {
            config,
            gpu: GPUResources::new(renderer, camera_buffer, config),
            cpu: CPUResources::new(config),
        }
    }

    pub fn set_view_box(&mut self, view_planes: &[Plane; 6]) {
        let mut frustum_aabb = Frustum::aabb(view_planes);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();
        self.cpu.update_view_delta(frustum_aabb);
    }

    pub fn compute_chunk_writes<'a>(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
        chunks: impl Iterator<Item = &'a VoxelChunk>,
    ) {
        self.cpu.prepare_chunk_writes(chunks);
        if self.cpu.chunks_write.is_empty() {
            return;
        }
        renderer.write_buffer(
            &self.gpu.chunks_write_buffer,
            0,
            bytemuck::cast_slice(&self.cpu.chunks_write),
        );
        compute_pass.set_pipeline(&self.gpu.chunk_write_pipeline);
        compute_pass.set_bind_group(0, &self.gpu.chunk_write_bind_group, &[]);
        let batch_size = self.cpu.chunks_write.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_2d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);
    }

    pub fn compute_chunk_visibility_and_meshing(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        self.cpu.allocate_view_delta();
        if self.cpu.view_chunks.cpu_dirty() {
            let view_write_delta = self.cpu.view_chunks.sync_delta();
            let batch_size = view_write_delta.len() as u32;
            renderer.write_buffer(
                &self.gpu.view_write_buffer,
                0,
                bytemuck::cast_slice(view_write_delta),
            );
            // write view candidates delta
            compute_pass.set_pipeline(&self.gpu.view_chunks_write_pipeline);
            compute_pass.set_bind_group(0, &self.gpu.view_chunks_write_bg, &[]);
            compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));

            let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_1d);
            compute_pass.dispatch_workgroups(wg_count, 1, 1);
        }

        // reset indirect args
        let dispatch_indirect = GPUDispatchIndirectArgsAtomic::new(0, 1, 1);
        let packed_indirect_args = GPUPackedIndirectArgsAtomic::new(0u32, dispatch_indirect);
        renderer.write_buffer(
            &self.gpu.packed_indirect_buffer,
            0,
            packed_indirect_args.as_bytes(),
        );

        // update mdi args and meshing queue
        compute_pass.set_pipeline(&self.gpu.mdi_args_pipeline);
        compute_pass.set_bind_group(0, &self.gpu.mdi_args_bind_group, &[]);
        let batch_size = self.cpu.view_chunks.cpu_len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_2d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);

        // handle meshing queue
        compute_pass.set_pipeline(&self.gpu.meshing_pipeline);
        compute_pass.set_bind_group(0, &self.gpu.meshing_bind_group, &[]);
        compute_pass.dispatch_workgroups_indirect(&self.gpu.packed_indirect_buffer, 4 * 4);
    }

    pub fn render_chunks(&mut self, renderer: &Renderer<'_>, render_pass: &mut RenderPass) {
        if self.cpu.view_chunks.cpu_empty() {
            return;
        }
        render_pass.set_bind_group(1, &self.gpu.render_bind_group, &[]);
        render_pass.multi_draw_indirect_count(
            &renderer.indirect_buffer,
            0,
            &self.gpu.packed_indirect_buffer,
            0,
            self.config.max_indirect_count,
        );
    }

    pub fn render_bgl(&self) -> &BindGroupLayout {
        &self.gpu.render_bind_group_layout
    }
}
