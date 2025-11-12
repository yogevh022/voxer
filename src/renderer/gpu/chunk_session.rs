use crate::compute::geo::{AABB, Frustum, Plane};
use crate::compute::num::{MaybeUsize, ceil_div};
use crate::renderer::gpu::chunk_session_mesh_data::{TRANSPARENT_LAYER_BLOCKS, chunk_mesh_data};
use crate::renderer::gpu::chunk_session_resources::GpuChunkSessionResources;
use crate::renderer::gpu::chunk_session_shader_types::{
    GPUChunkMeshEntry, GPUPackedIndirectArgsAtomic, GPUVoxelChunkHeader,
};
use crate::renderer::gpu::chunk_session_types::{ChunkMeshState, MeshStateError};
use crate::renderer::gpu::{
    GPUChunkMeshEntryWrite, GPUDispatchIndirectArgsAtomic, GPUVoxelChunk, GPUVoxelFaceData,
};
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::{Renderer, resources};
use crate::world::block::VoxelBlock;
use crate::world::chunk::VoxelChunk;
use crate::world::{CHUNK_DIM, VoxelChunkAdjBlocks};
use glam::IVec3;
use rustc_hash::FxHashMap;
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

struct GpuState {
    mesh_allocator: SubAllocator,
    chunk_cache: SlabMap<IVec3, ChunkMeshState>,

    chunk_buffer: VxBuffer,
    visible_candidates_buffer: VxBuffer,
    visible_candidates_write_buffer: VxBuffer,
    write_batch_buffer: VxBuffer,
    meshing_batch_buffer: VxBuffer,
    voxel_face_buffer: VxBuffer,
    packed_indirect_buffer: VxBuffer,

    mdi_args_pipeline: ComputePipeline,
    mdi_args_bind_group: BindGroup,

    chunk_write_pipeline: ComputePipeline,
    chunk_write_bind_group: BindGroup,

    view_candidate_write_pipeline: ComputePipeline,
    view_candidate_write_bg: BindGroup,

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

        let view_candidates_buffer = renderer.device.create_vx_buffer::<GPUChunkMeshEntry>(
            "Chunk Manager View Candidates Buffer",
            config.max_chunks, // fixme overkill but cheap anyway?
            BufferUsages::STORAGE,
        );

        let view_candidates_write_buffer =
            renderer.device.create_vx_buffer::<GPUChunkMeshEntryWrite>(
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
            meshing_batch_buffer.buffer_size,
            chunk_buffer.buffer_size,
            view_candidates_buffer.buffer_size,
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
                view_candidates_buffer.as_entire_binding(),
                BindingResource::TextureView(&renderer.depth.mip_texture_array_view),
                camera_buffer.as_entire_binding(),
            ]),
        });

        let chunk_write_bgl = GpuChunkSessionResources::write_bgl(
            &renderer.device,
            write_batch_buffer.buffer_size,
            chunk_buffer.buffer_size,
        );
        let chunk_write_pipeline =
            GpuChunkSessionResources::chunk_write_pipeline(&renderer.device, &[&chunk_write_bgl]);
        let chunk_write_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Write Bind Group"),
            layout: &chunk_write_bgl,
            entries: &resources::utils::bind_entries([
                write_batch_buffer.as_entire_binding(),
                chunk_buffer.as_entire_binding(),
            ]),
        });

        let visible_candidate_write_bgl = GpuChunkSessionResources::write_bgl(
            &renderer.device,
            view_candidates_write_buffer.buffer_size,
            view_candidates_buffer.buffer_size,
        );
        let view_candidate_write_pipeline = GpuChunkSessionResources::view_candidate_write_pipeline(
            &renderer.device,
            &[&visible_candidate_write_bgl],
        );
        let view_candidate_write_bg = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Visible Candidate Write Bind Group"),
            layout: &visible_candidate_write_bgl,
            entries: &resources::utils::bind_entries([
                view_candidates_write_buffer.as_entire_binding(),
                view_candidates_buffer.as_entire_binding(),
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
            visible_candidates_buffer: view_candidates_buffer,
            visible_candidates_write_buffer: view_candidates_write_buffer,
            packed_indirect_buffer,

            mdi_args_pipeline,
            mdi_args_bind_group,
            chunk_write_pipeline,
            chunk_write_bind_group,
            view_candidate_write_pipeline,
            view_candidate_write_bg,
            meshing_pipeline,
            meshing_bind_group,
            render_bind_group,
            render_bind_group_layout,
        }
    }

    fn cache_chunk(&mut self, chunk: &VoxelChunk) -> GPUVoxelChunk {
        let mesh_state = ChunkMeshState::Uninitialized;
        let chunk_index = self.chunk_cache.insert(chunk.position, mesh_state);
        let header = GPUVoxelChunkHeader::new(chunk_index as u32, chunk.position);
        GPUVoxelChunk::new_uninit(header, chunk.blocks)
    }

    fn drop_chunk(&mut self, position: &IVec3) {
        let (_, mesh_state) = self.chunk_cache.remove(position).unwrap();
        if let ChunkMeshState::Meshed(mesh_entry) = mesh_state {
            let alloc = mesh_entry.face_alloc;
            self.mesh_allocator.deallocate(alloc).unwrap();
        }
    }

    fn drop_chunk_mesh(&mut self, position: &IVec3) {
        let (_, mesh_state) = self.chunk_cache.get_mut(position).unwrap();
        let ChunkMeshState::Meshed(mesh_entry) = mesh_state else {
            unreachable!("chunk mesh state is not meshed");
        };

        let alloc = mesh_entry.face_alloc;
        self.mesh_allocator.deallocate(alloc).unwrap();
        mesh_state.set_as_unmeshed();
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

enum ArenaSwapResult {
    Swap(usize, usize),
    Truncate(usize, usize),
}

struct CpuState {
    chunk_adj_cache: FxHashMap<IVec3, VoxelChunkAdjBlocks>,
    chunk_to_view_index: Vec<MaybeUsize>,
    last_view_aabb: AABB,
    chunk_write_buff: Vec<GPUVoxelChunk>,
    view_candidates_buff: Vec<GPUChunkMeshEntry>,
    view_candidates_write_buff: Vec<GPUChunkMeshEntryWrite>,
    view_candidates_drop_buff: Vec<usize>,
    view_new_boxes_buff: Vec<AABB>,
    view_out_boxes_buff: Vec<AABB>,
}

impl CpuState {
    fn new(config: GpuChunkSessionConfig) -> Self {
        let mut chunk_adj_cache = FxHashMap::default();
        chunk_adj_cache.reserve(config.max_chunks);
        let max_view_count_est = config.max_chunks / 4; // very conservative
        Self {
            chunk_adj_cache,
            chunk_to_view_index: vec![MaybeUsize::default(); config.max_chunks],
            last_view_aabb: AABB::zero(),

            chunk_write_buff: Vec::with_capacity(config.max_chunks),
            view_candidates_buff: Vec::with_capacity(config.max_chunks),
            view_candidates_write_buff: Vec::with_capacity(max_view_count_est),
            view_candidates_drop_buff: Vec::with_capacity(max_view_count_est),
            view_new_boxes_buff: Vec::with_capacity(6),
            view_out_boxes_buff: Vec::with_capacity(6),
        }
    }

    fn view_box_delta(&mut self, frustum_aabb: AABB) -> (*const Vec<AABB>, *const Vec<AABB>) {
        AABB::sym_diff_out(
            frustum_aabb,
            self.last_view_aabb,
            &mut self.view_new_boxes_buff,
            &mut self.view_out_boxes_buff,
        );

        // SAFETY: these vectors are exclusively mutated here, by the above function
        let new_boxes = &self.view_new_boxes_buff as *const Vec<AABB>;
        let out_boxes = &self.view_out_boxes_buff as *const Vec<AABB>;

        self.last_view_aabb = frustum_aabb;

        (new_boxes, out_boxes)
    }

    fn prepare_mesh_write_buff(&mut self) {
        // asc sort by view index
        self.view_candidates_drop_buff
            .sort_by_key(|&k| self.chunk_to_view_index[k].unwrap());
        // clear write buffer
        self.view_candidates_write_buff.clear();
    }

    fn view_candidates_compaction_writes(&mut self) {
        if self.view_candidates_drop_buff.is_empty() {
            return;
        }
        let mut drop_count = self.view_candidates_drop_buff.len();
        let mut view_count = self.view_candidates_buff.len();
        let mut current_drop_idx = 0;
        while current_drop_idx < drop_count {
            // fixme redundant definition in some iterations
            let current_drop_chunk_idx = self.view_candidates_drop_buff[current_drop_idx];
            let current_drop_view_idx = self.chunk_to_view_index[current_drop_chunk_idx].unwrap();
            if current_drop_view_idx >= view_count {
                break;
            }
            let last_drop_ch_idx = self.view_candidates_drop_buff[drop_count - 1];
            let last_drop_view_idx = self.chunk_to_view_index[last_drop_ch_idx].unwrap();
            let swap_view_idx = view_count - 1;

            if last_drop_view_idx == swap_view_idx {
                // drop last view candidate
                self.chunk_to_view_index[last_drop_ch_idx] = MaybeUsize::default();
                self.view_candidates_buff.pop();
                drop_count -= 1;
                view_count -= 1;
                continue;
            }

            // swap logic
            self.chunk_to_view_index[current_drop_chunk_idx] = MaybeUsize::default();
            let swap_mesh_entry = self.view_candidates_buff.pop().unwrap();
            let swap_mesh_entry_write =
                GPUChunkMeshEntryWrite::new(swap_mesh_entry, swap_view_idx as u32);
            let swap_mesh_entry_idx = swap_mesh_entry.index as usize;
            self.chunk_to_view_index[swap_mesh_entry_idx] = MaybeUsize::new(swap_view_idx);
            self.view_candidates_buff[current_drop_view_idx] = swap_mesh_entry;
            self.view_candidates_write_buff.push(swap_mesh_entry_write);
            view_count -= 1;
            current_drop_idx += 1;
            continue;
        }
    }

    fn view_candidates_entry_write(&mut self, mesh_entry: GPUChunkMeshEntry) {
        let mesh_write_index = self.view_candidates_buff.len();
        let mesh_chunk_index = mesh_entry.index as usize;
        let mesh_entry_write = GPUChunkMeshEntryWrite::new(mesh_entry, mesh_write_index as u32);
        self.chunk_to_view_index[mesh_chunk_index] = MaybeUsize::new(mesh_write_index);
        self.view_candidates_buff.push(mesh_entry);
        self.view_candidates_write_buff.push(mesh_entry_write);
    }

    fn cache_adj_blocks(&mut self, chunk: &VoxelChunk) {
        let as_adj = chunk.blocks_as_adj();
        self.chunk_adj_cache.insert(chunk.position, as_adj);
    }

    fn drop_adj_blocks(&mut self, position: &IVec3) {
        self.chunk_adj_cache.remove(position);
    }

    fn adj_or_transparent(&self, position: &IVec3, index: usize) -> [[VoxelBlock; 16]; 16] {
        self.chunk_adj_cache
            .get(position)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |adj| adj[index])
    }

    fn adj_blocks_of(&self, position: &IVec3) -> VoxelChunkAdjBlocks {
        let px = IVec3::new(position.x + 1, position.y, position.z);
        let py = IVec3::new(position.x, position.y + 1, position.z);
        let pz = IVec3::new(position.x, position.y, position.z + 1);

        let mx = IVec3::new(position.x - 1, position.y, position.z);
        let my = IVec3::new(position.x, position.y - 1, position.z);
        let mz = IVec3::new(position.x, position.y, position.z - 1);

        let adj = [
            self.adj_or_transparent(&px, 0),
            self.adj_or_transparent(&py, 1),
            self.adj_or_transparent(&pz, 2),
            self.adj_or_transparent(&mx, 3),
            self.adj_or_transparent(&my, 4),
            self.adj_or_transparent(&mz, 5),
        ];

        adj
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

    fn cache_chunk(&mut self, chunk: &VoxelChunk) -> GPUVoxelChunk {
        self.cpu_state.cache_adj_blocks(chunk);
        self.gpu_state.cache_chunk(chunk)
    }

    pub fn drop_chunk(&mut self, position: &IVec3) {
        self.cpu_state.drop_adj_blocks(position);
        self.gpu_state.drop_chunk(position);
    }

    pub fn prepare_chunk_writes<'a>(&mut self, chunks: impl Iterator<Item = &'a VoxelChunk>) {
        let write_batch: &mut Vec<GPUVoxelChunk> =
            unsafe { &mut *((&mut self.cpu_state.chunk_write_buff) as *mut _) };
        write_batch.clear();
        for chunk in chunks.take(self.config.max_write_count) {
            if self.gpu_state.is_chunk_cached(&chunk.position) {
                self.drop_chunk(&chunk.position);
            }
            let gpu_chunk = self.cache_chunk(chunk);
            write_batch.push(gpu_chunk);
        }

        for gpu_chunk in write_batch.iter_mut() {
            let blocks = unsafe { std::mem::transmute(&gpu_chunk.content) };
            let adj_blocks = self.cpu_state.adj_blocks_of(&gpu_chunk.header.position());
            let mesh_meta = chunk_mesh_data(blocks, &adj_blocks);
            let mesh_state = ChunkMeshState::new_unmeshed(mesh_meta);
            gpu_chunk.adj_content = unsafe { std::mem::transmute(adj_blocks) };
            let index = gpu_chunk.header.index as usize;
            unsafe { self.gpu_state.chunk_cache.set_index(index, mesh_state) };
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

        let (to_alloc_ptr, to_dealloc_ptr) = self.cpu_state.view_box_delta(frustum_aabb);
        let to_alloc = unsafe { &*to_alloc_ptr };
        let to_dealloc = unsafe { &*to_dealloc_ptr };

        self.cpu_state.view_candidates_drop_buff.clear();
        for view_drop_box in to_dealloc.iter() {
            view_drop_box.discrete_points(|ch_pos| {
                if let Some((chunk_index, mesh_state)) = self.gpu_state.chunk_cache.get(&ch_pos) {
                    let ChunkMeshState::Meshed(_) = mesh_state else {
                        return;
                    };
                    self.gpu_state.drop_chunk_mesh(&ch_pos);
                    self.cpu_state.view_candidates_drop_buff.push(chunk_index);
                }
            });
        }

        self.cpu_state.prepare_mesh_write_buff();
        self.cpu_state.view_candidates_compaction_writes();

        for view_new_box in to_alloc.iter() {
            view_new_box.discrete_points(|ch_pos| {
                match self.gpu_state.prepare_chunk_mesh_entry(&ch_pos) {
                    Ok(mesh_entry) => self.cpu_state.view_candidates_entry_write(mesh_entry),
                    Err(MeshStateError::Missing) => missing_chunk(ch_pos),
                    Err(_) => (),
                }
            })
        }

        // self.cpu_state.view_candidates.clear();
        // let mut i = 0;
        // frustum_aabb.discrete_points(|ch_pos| {
        //     match self.gpu_state.prepare_chunk_mesh_entry(&ch_pos) {
        //         Ok(chunk_mesh_entry) => {
        //             let write_entry = GPUChunkMeshEntryWrite::new(chunk_mesh_entry, i);
        //             self.cpu_state.view_candidates_write_batch.push(write_entry);
        //             self.cpu_state.view_candidates.push(chunk_mesh_entry);
        //             i += 1;
        //         }
        //         Err(MeshStateError::Missing) => missing_chunk(ch_pos),
        //         Err(_) => (),
        //     };
        // });
    }

    pub fn compute_chunk_writes(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        if self.cpu_state.chunk_write_buff.is_empty() {
            return;
        }
        renderer.write_buffer(
            &self.gpu_state.write_batch_buffer,
            0,
            bytemuck::cast_slice(&self.cpu_state.chunk_write_buff),
        );
        compute_pass.set_pipeline(&self.gpu_state.chunk_write_pipeline);
        compute_pass.set_bind_group(0, &self.gpu_state.chunk_write_bind_group, &[]);
        let batch_size = self.cpu_state.chunk_write_buff.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_2d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);
    }

    pub fn compute_chunk_visibility_and_meshing(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        if self.cpu_state.view_candidates_buff.is_empty() {
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
            &self.gpu_state.visible_candidates_write_buffer,
            0,
            bytemuck::cast_slice(&self.cpu_state.view_candidates_write_buff),
        );
        // write view candidates delta
        compute_pass.set_pipeline(&self.gpu_state.view_candidate_write_pipeline);
        compute_pass.set_bind_group(0, &self.gpu_state.view_candidate_write_bg, &[]);
        let batch_size = self.cpu_state.view_candidates_write_buff.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_1d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);

        // update mdi args and meshing queue
        compute_pass.set_pipeline(&self.gpu_state.mdi_args_pipeline);
        compute_pass.set_bind_group(0, &self.gpu_state.mdi_args_bind_group, &[]);
        let batch_size = self.cpu_state.view_candidates_buff.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = ceil_div(batch_size, self.config.max_workgroup_size_2d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);

        // handle meshing queue
        compute_pass.set_pipeline(&self.gpu_state.meshing_pipeline);
        compute_pass.set_bind_group(0, &self.gpu_state.meshing_bind_group, &[]);
        compute_pass.dispatch_workgroups_indirect(&self.gpu_state.packed_indirect_buffer, 4 * 4);
    }

    pub fn render_chunks(&mut self, renderer: &Renderer<'_>, render_pass: &mut RenderPass) {
        if self.cpu_state.view_candidates_buff.is_empty() {
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
