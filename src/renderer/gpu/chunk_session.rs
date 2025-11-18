use crate::compute::geo::{AABB, Frustum, Plane};
use crate::compute::num::{MaybeUsize, ceil_div};
use crate::compute::utils::fxmap_with_capacity;
use crate::renderer::gpu::chunk_session_mesh_data::{TRANSPARENT_LAYER_BLOCKS, chunk_mesh_data};
use crate::renderer::gpu::chunk_session_resources::GpuChunkSessionResources;
use crate::renderer::gpu::chunk_session_shader_types::{
    GPUChunkMeshEntry, GPUPackedIndirectArgsAtomic, GPUVoxelChunkHeader,
};
use crate::renderer::gpu::chunk_session_types::ChunkMeshState;
use crate::renderer::gpu::{
    GPUChunkMeshEntryWrite, GPUDispatchIndirectArgsAtomic, GPUVoxelChunk, GPUVoxelFaceData,
};
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::{Renderer, resources};
use crate::world::block::VoxelBlock;
use crate::world::chunk::VoxelChunk;
use crate::world::{CHUNK_DIM, VoxelChunkAdjBlocks};
use glam::{IVec3, Vec3};
use rustc_hash::{FxHashMap, FxHashSet};
use slabmap::SlabMap;
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
    view_box: AABB,
    mesh_allocator: SubAllocator,
    chunks: SlabMap<IVec3, ChunkMeshState>,
    chunks_adj: FxHashMap<IVec3, VoxelChunkAdjBlocks>,

    chunks_write: Vec<GPUVoxelChunk>,
    view: Vec<GPUChunkMeshEntry>,
    view_write: Vec<GPUChunkMeshEntryWrite>,
    view_drop: Vec<usize>, // chunk index

    chunk_to_view_index: Vec<MaybeUsize>,
    view_new_positions: Vec<IVec3>,
    view_new_boxes: Vec<AABB>,
    view_out_boxes: Vec<AABB>,
}

impl CPUResources {
    fn new(config: GpuChunkSessionConfig) -> Self {
        let max_view_count_est = config.max_chunks / 4; // very conservative
        Self {
            view_box: AABB::zero(),
            mesh_allocator: SubAllocator::new(config.max_face_count as u32),
            chunks: SlabMap::with_capacity(config.max_chunks),
            chunks_adj: fxmap_with_capacity(config.max_chunks),

            chunks_write: Vec::with_capacity(config.max_chunks),
            view: Vec::with_capacity(config.max_chunks),
            view_write: Vec::with_capacity(max_view_count_est),
            view_drop: Vec::with_capacity(max_view_count_est),

            chunk_to_view_index: vec![MaybeUsize::default(); config.max_chunks],
            view_new_positions: Vec::with_capacity(max_view_count_est),
            view_new_boxes: Vec::with_capacity(6),
            view_out_boxes: Vec::with_capacity(6),
        }
    }

    fn insert_chunk(&mut self, chunk: &VoxelChunk) -> GPUVoxelChunk {
        // cache adjacent blocks
        let as_adj = chunk.blocks_as_adj();
        self.chunks_adj.insert(chunk.position, as_adj);

        // deallocate mesh if exists
        self.deallocate_mesh(&chunk.position);

        // add to view write if on screen
        if self.view_box.contains_point(chunk.position.as_vec3()) {
            self.view_new_positions.push(chunk.position);
        }

        // cache chunk
        let mesh_state = ChunkMeshState::Uninitialized;
        let chunk_index = self.chunks.insert(chunk.position, mesh_state);
        let header = GPUVoxelChunkHeader::new(chunk_index as u32, chunk.position);
        GPUVoxelChunk::new_uninit(header, chunk.blocks)
    }

    fn drop_chunk(&mut self, position: &IVec3) {
        self.chunks_adj.remove(position);
        let (chunk_index, mut mesh_state) = self.chunks.remove(position).unwrap();
        self.deallocate_mesh_ptr(chunk_index, &mut mesh_state);
    }

    fn deallocate_mesh(&mut self, chunk_position: &IVec3) {
        if let Some((chunk_index, mesh_state)) = self.chunks.get_mut(&chunk_position) {
            let ms_ptr = mesh_state as *mut _;
            self.deallocate_mesh_ptr(chunk_index, ms_ptr);
        }
    }

    fn deallocate_mesh_ptr(&mut self, chunk_index: usize, mesh_state_ptr: *mut ChunkMeshState) {
        let mesh_state = unsafe {
            // SAFETY: mesh_state_ptr is always valid and created just before this call
            // to avoid borrow checker false alarm
            &mut *mesh_state_ptr
        };
        match mesh_state {
            ChunkMeshState::Allocated(mesh_entry) => {
                let alloc = mesh_entry.face_alloc;
                self.mesh_allocator.deallocate(alloc).unwrap();
                mesh_state.set_unallocated();
                self.view_drop.push(chunk_index);
            }
            ChunkMeshState::AllocatedEmpty => mesh_state.set_unallocated(),
            _ => (),
        }
    }

    fn allocate_mesh(&mut self, chunk_position: &IVec3) {
        if let Some((chunk_index, mesh_state)) = self.chunks.get_mut(&chunk_position) {
            let ChunkMeshState::Unallocated(mesh_entry) = mesh_state else {
                return;
            };
            if self.chunk_to_view_index[chunk_index].is_some() {
                return;
            }

            let face_count = mesh_entry.face_count();
            if face_count > 0 {
                let alloc_result = self.mesh_allocator.allocate(face_count);
                let alloc = alloc_result.unwrap(); // .map_err(|_| MeshStateError::FailedAllocation)?;
                mesh_state.set_allocated(chunk_index as u32, alloc);
                let view_entry = mesh_state.meshing_flagged_entry();
                self.view_push(view_entry);
            } else {
                mesh_state.set_empty();
            }
        }
    }

    fn initialize_chunk_writes(&mut self) {
        for i in 0..self.chunks_write.len() {
            let adj_blocks = self.adj_blocks_of(&self.chunks_write[i].header.position());
            let gpu_chunk = &mut self.chunks_write[i];
            let blocks = unsafe { std::mem::transmute(&gpu_chunk.content) };
            let index = gpu_chunk.header.index as usize;
            gpu_chunk.adj_content = unsafe { std::mem::transmute(adj_blocks) };

            let mesh_meta = chunk_mesh_data(blocks, &adj_blocks);
            let mesh_state = ChunkMeshState::new(mesh_meta);
            unsafe { self.chunks.set_index(index, mesh_state) };
        }
    }

    fn deallocate_view_out_delta(&mut self) {
        for i in 0..self.view_out_boxes.len() {
            let view_drop_box = self.view_out_boxes[i];
            view_drop_box.discrete_points(|ch_pos| {
                self.deallocate_mesh(&ch_pos);
            });
        }
    }

    fn allocate_view_new_delta(&mut self) {
        for i in 0..self.view_new_boxes.len() {
            let view_new_box = self.view_new_boxes[i];
            view_new_box.discrete_points(|ch_pos| {
                self.allocate_mesh(&ch_pos);
            });
        }
    }

    fn allocate_view_new_positions(&mut self) {
        for i in 0..self.view_new_positions.len() {
            let ch_pos = self.view_new_positions[i];
            self.allocate_mesh(&ch_pos);
        }
        self.view_new_positions.clear();
    }

    fn view_box_delta(&mut self, new_view_box: AABB) {
        AABB::sym_diff_out(
            new_view_box,
            self.view_box,
            &mut self.view_new_boxes,
            &mut self.view_out_boxes,
        );
    }

    fn view_write_prepare(&mut self) {
        // asc sort by view index
        self.view_drop
            .sort_by_key(|&idx| self.chunk_to_view_index[idx].unwrap());
        // clear write buffer
        self.view_write.clear();
    }

    fn view_pop(&mut self, chunk_index: usize) -> GPUChunkMeshEntry {
        self.chunk_to_view_index[chunk_index] = MaybeUsize::default();
        self.view.pop().unwrap()
    }

    fn view_push(&mut self, entry: GPUChunkMeshEntry) {
        let view_index = self.view.len();
        let chunk_index = entry.index as usize;
        self.chunk_to_view_index[chunk_index] = MaybeUsize::new(view_index);
        self.view.push(entry);
        let entry_write = GPUChunkMeshEntryWrite::new(entry, view_index as u32);
        self.view_write.push(entry_write);
    }

    fn view_swap_remove(&mut self, chunk_index: usize, view_index: usize) {
        let swap_entry = self.view.pop().unwrap();
        let swap_chunk_index = swap_entry.index as usize;
        self.chunk_to_view_index[chunk_index] = MaybeUsize::default();
        self.chunk_to_view_index[swap_chunk_index] = MaybeUsize::new(view_index);
        self.view[view_index] = swap_entry;
        let write_entry = GPUChunkMeshEntryWrite::new(swap_entry, view_index as u32);
        self.view_write.push(write_entry);
    }

    fn buffer_view_drop_and_defrag(&mut self) {
        if self.view_drop.is_empty() {
            return;
        }
        let mut drop_min = 0;
        let mut drop_max = self.view_drop.len();
        let mut view_max = self.view.len();
        while drop_min < drop_max {
            let drop_min_chunk = self.view_drop[drop_min];
            let drop_min_view = self.chunk_to_view_index[drop_min_chunk].unwrap();
            if drop_min_view >= view_max {
                break;
            }
            let drop_max_chunk = self.view_drop[drop_max - 1];
            let drop_max_view = self.chunk_to_view_index[drop_max_chunk].unwrap();

            if drop_max_view == view_max - 1 {
                self.view_pop(drop_max_chunk);
                drop_max -= 1;
            } else {
                self.view_swap_remove(drop_min_chunk, drop_min_view);
                drop_min += 1;
            }
            view_max -= 1;
        }
        self.view_drop.clear();
    }

    fn adj_or_transparent(&self, position: &IVec3, index: usize) -> [[VoxelBlock; 16]; 16] {
        self.chunks_adj
            .get(position)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |adj| adj[index])
    }

    fn adj_blocks_of(&self, position: &IVec3) -> VoxelChunkAdjBlocks {
        [
            self.adj_or_transparent(&IVec3::new(position.x + 1, position.y, position.z), 0), // px
            self.adj_or_transparent(&IVec3::new(position.x, position.y + 1, position.z), 1), // py
            self.adj_or_transparent(&IVec3::new(position.x, position.y, position.z + 1), 2), // pz
            self.adj_or_transparent(&IVec3::new(position.x - 1, position.y, position.z), 3), // mx
            self.adj_or_transparent(&IVec3::new(position.x, position.y - 1, position.z), 4), // my
            self.adj_or_transparent(&IVec3::new(position.x, position.y, position.z - 1), 5), // mz
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

    pub(crate) fn drop_chunk(&mut self, position: &IVec3) {
        self.cpu.drop_chunk(position);
    }

    pub fn prepare_chunk_writes<'a>(&mut self, chunks: impl Iterator<Item = &'a VoxelChunk>) {
        self.cpu.chunks_write.clear();
        for chunk in chunks.take(self.config.max_write_count) {
            let gpu_chunk = self.cpu.insert_chunk(chunk);
            self.cpu.chunks_write.push(gpu_chunk);
        }
        self.cpu.initialize_chunk_writes();
    }

    pub fn set_view_box(&mut self, view_planes: &[Plane; 6]) {
        let mut frustum_aabb = Frustum::aabb(view_planes);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();
        self.cpu.view_box_delta(frustum_aabb);
        self.cpu.view_box = frustum_aabb;
    }

    pub fn prepare_chunk_visibility(&mut self) {
        // prepare drop buffer
        self.cpu.deallocate_view_out_delta();
        self.cpu.view_write_prepare();

        // update drop chunks changes (zero fragmentation algo) and write delta to buffer
        self.cpu.buffer_view_drop_and_defrag();

        // update new chunks changes and write delta to buffer
        self.cpu.allocate_view_new_delta();
        self.cpu.allocate_view_new_positions();
    }

    pub fn compute_chunk_writes(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
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
        if self.cpu.view.is_empty() {
            return;
        }
        // reset indirect args
        let dispatch_indirect = GPUDispatchIndirectArgsAtomic::new(0, 1, 1);
        let packed_indirect_args = GPUPackedIndirectArgsAtomic::new(0u32, dispatch_indirect);
        renderer.write_buffer(
            &self.gpu.packed_indirect_buffer,
            0,
            packed_indirect_args.as_bytes(),
        );
        if !self.cpu.view_write.is_empty() {
            renderer.write_buffer(
                &self.gpu.view_write_buffer,
                0,
                bytemuck::cast_slice(&self.cpu.view_write),
            );
            // write view candidates delta
            compute_pass.set_pipeline(&self.gpu.view_chunks_write_pipeline);
            compute_pass.set_bind_group(0, &self.gpu.view_chunks_write_bg, &[]);
            let batch_size = self.cpu.view_write.len() as u32;
            compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
            let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_1d);
            compute_pass.dispatch_workgroups(wg_count, 1, 1);
        }

        // update mdi args and meshing queue
        compute_pass.set_pipeline(&self.gpu.mdi_args_pipeline);
        compute_pass.set_bind_group(0, &self.gpu.mdi_args_bind_group, &[]);
        let batch_size = self.cpu.view.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_2d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);

        // handle meshing queue
        compute_pass.set_pipeline(&self.gpu.meshing_pipeline);
        compute_pass.set_bind_group(0, &self.gpu.meshing_bind_group, &[]);
        compute_pass.dispatch_workgroups_indirect(&self.gpu.packed_indirect_buffer, 4 * 4);
    }

    pub fn render_chunks(&mut self, renderer: &Renderer<'_>, render_pass: &mut RenderPass) {
        if self.cpu.view.is_empty() {
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
