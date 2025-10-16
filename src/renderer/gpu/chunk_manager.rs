use crate::compute::geo::{Frustum, Plane};
use crate::renderer::gpu::chunk_entry::{
    GPUChunkMeshEntry, GPUPackedIndirectArgsAtomic, GPUVoxelChunkHeader,
};
use crate::renderer::gpu::gpu_state_types::{ChunkMeshState, MeshStateError};
use crate::renderer::gpu::{GPUDispatchIndirectArgsAtomic, GPUVoxelChunk, GPUVoxelFaceData};
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::{Renderer, resources};
use crate::world::types::{CHUNK_DIM, Chunk};
use glam::IVec3;
use slabmap::SlabMap;
use suballoc::SubAllocator;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferSize, BufferUsages, ComputePass,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor,
    PushConstantRange, RenderPass, ShaderStages,
};

pub struct ChunkManagerConfig {
    pub max_indirect_count: u32,
    pub max_workgroup_size_1d: u32,
    pub max_workgroup_size_2d: u32,
    pub max_chunks: usize,
    pub max_write_count: usize,
    pub max_face_count: usize,
}

pub struct ChunkManager {
    config: ChunkManagerConfig,

    // gpu state
    gpu_mesh_allocator: SubAllocator,
    gpu_cached: SlabMap<IVec3, ChunkMeshState>,

    write_batch: Vec<GPUVoxelChunk>,
    aabb_visible_batch: Vec<GPUChunkMeshEntry>,

    // buffers
    chunk_buffer: VxBuffer,
    aabb_visible_buffer: VxBuffer,
    write_batch_buffer: VxBuffer,
    meshing_batch_buffer: VxBuffer,
    voxel_face_buffer: VxBuffer,
    packed_indirect_buffer: VxBuffer,

    culled_mdi_args_pipeline: ComputePipeline,
    culled_mdi_args_bind_group: BindGroup,

    write_pipeline: ComputePipeline,
    write_bind_group: BindGroup,

    meshing_pipeline: ComputePipeline,
    meshing_bind_group: BindGroup,

    render_bind_group: BindGroup,
    pub(crate) render_bind_group_layout: BindGroupLayout,
}

impl ChunkManager {
    pub fn new(
        renderer: &Renderer<'_>,
        camera_buffer: &VxBuffer,
        config: ChunkManagerConfig,
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

        let culled_mdi_args_bgl = culled_mdi_args_bgl(
            &renderer.device,
            renderer.indirect_buffer.buffer_size,
            packed_indirect_buffer.buffer_size,
            meshing_batch_buffer.buffer_size,
            chunk_buffer.buffer_size,
            aabb_visible_buffer.buffer_size,
            camera_buffer.buffer_size,
        );
        let culled_mdi_args_pipeline =
            culled_mdi_args_pipeline(&renderer.device, &[&culled_mdi_args_bgl]);
        let culled_mdi_args_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Culled MDI Args Bind Group"),
            layout: &culled_mdi_args_bgl,
            entries: &resources::utils::bind_entries([
                renderer.indirect_buffer.as_entire_binding(),
                packed_indirect_buffer.as_entire_binding(),
                meshing_batch_buffer.as_entire_binding(),
                chunk_buffer.as_entire_binding(),
                aabb_visible_buffer.as_entire_binding(),
                camera_buffer.as_entire_binding(),
            ]),
        });

        let write_bgl = chunk_write_bgl(
            &renderer.device,
            chunk_buffer.buffer_size,
            write_batch_buffer.buffer_size,
        );
        let write_pipeline = chunk_write_pipeline(&renderer.device, &[&write_bgl]);
        let write_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Write Bind Group"),
            layout: &write_bgl,
            entries: &resources::utils::bind_entries([
                chunk_buffer.as_entire_binding(),
                write_batch_buffer.as_entire_binding(),
            ]),
        });

        let meshing_bgl = chunk_meshing_bgl(
            &renderer.device,
            chunk_buffer.buffer_size,
            voxel_face_buffer.buffer_size,
            meshing_batch_buffer.buffer_size,
        );
        let meshing_pipeline = chunk_meshing_pipeline(&renderer.device, &[&meshing_bgl]);
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
            chunk_render_bind_group(&renderer.device, camera_buffer, &voxel_face_buffer);

        Self {
            gpu_mesh_allocator: SubAllocator::new(config.max_face_count as u32),
            gpu_cached: SlabMap::with_capacity(config.max_chunks),

            write_batch: Vec::with_capacity(config.max_chunks),
            aabb_visible_batch: Vec::with_capacity(1024), // todo decent approximation for max visible chunks

            chunk_buffer,
            write_batch_buffer,
            voxel_face_buffer,
            meshing_batch_buffer,
            aabb_visible_buffer,
            packed_indirect_buffer,

            culled_mdi_args_pipeline,
            culled_mdi_args_bind_group,
            write_pipeline,
            write_bind_group,
            meshing_pipeline,
            meshing_bind_group,
            render_bind_group,
            render_bind_group_layout,

            config,
        }
    }

    fn cache_chunk(&mut self, chunk: &Chunk) -> GPUVoxelChunk {
        let mesh_meta = chunk.mesh_meta.unwrap();
        let mesh_state = ChunkMeshState::new_unmeshed(mesh_meta);
        let chunk_index = self.gpu_cached.insert(chunk.position, mesh_state);
        let header = GPUVoxelChunkHeader::new(chunk_index as u32, chunk.position);
        GPUVoxelChunk::new(header, chunk.adjacent_blocks, chunk.blocks)
    }

    fn is_chunk_cached(&self, position: &IVec3) -> bool {
        self.gpu_cached.get(position).is_some()
    }

    pub fn prepare_gpu_chunk_writes(&mut self, chunks: &[Chunk]) {
        debug_assert!(chunks.len() <= self.config.max_write_count);
        self.write_batch.clear();
        for chunk in chunks {
            if self.is_chunk_cached(&chunk.position) {
                self.drop_chunk(&chunk.position);
            }
            let gpu_chunk = self.cache_chunk(chunk);
            self.write_batch.push(gpu_chunk);
        }
    }

    pub fn encode_gpu_chunk_writes(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        if self.write_batch.is_empty() {
            return;
        }
        renderer.write_buffer(
            &self.write_batch_buffer,
            0,
            bytemuck::cast_slice(&self.write_batch),
        );
        compute_pass.set_pipeline(&self.write_pipeline);
        compute_pass.set_bind_group(0, &self.write_bind_group, &[]);
        let batch_size = self.write_batch.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = dispatch_count_1d(batch_size, self.config.max_workgroup_size_1d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);
    }

    fn chunk_mesh_entry(
        &mut self,
        chunk_position: &IVec3,
    ) -> Result<GPUChunkMeshEntry, MeshStateError> {
        match self.gpu_cached.get_mut(chunk_position) {
            Some((chunk_index, render_mesh_state)) => match render_mesh_state {
                ChunkMeshState::Meshed(mesh_entry) => Ok(*mesh_entry),
                ChunkMeshState::Unmeshed(unmeshed_entry) if unmeshed_entry.has_faces() => {
                    let allocation = self
                        .gpu_mesh_allocator
                        .allocate(unmeshed_entry.total_faces)
                        .map_err(|_| MeshStateError::FailedAllocation)?;
                    render_mesh_state.set_as_meshed(chunk_index as u32, allocation);
                    Ok(render_mesh_state.entry_flagged_to_mesh())
                }
                _ => Err(MeshStateError::Empty),
            },
            None => Err(MeshStateError::Missing),
        }
    }

    pub fn prepare_visible_chunks(
        &mut self,
        view_planes: &[Plane; 6],
        mut missing_chunk: impl FnMut(IVec3),
    ) {
        let mut frustum_aabb = Frustum::aabb(view_planes);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();

        self.aabb_visible_batch.clear();
        frustum_aabb.discrete_points(|ch_pos| {
            match self.chunk_mesh_entry(&ch_pos) {
                Ok(chunk_mesh_entry) => {
                    self.aabb_visible_batch.push(chunk_mesh_entry);
                }
                Err(MeshStateError::Missing) => missing_chunk(ch_pos),
                Err(_) => (),
            };
        });
    }

    pub fn encode_gpu_view_chunks(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        if self.aabb_visible_batch.is_empty() {
            return;
        }
        // reset indirect args
        let dispatch_indirect = GPUDispatchIndirectArgsAtomic::new(0, 1, 1);
        let packed_indirect_args = GPUPackedIndirectArgsAtomic::new(0u32, dispatch_indirect);
        renderer.write_buffer(
            &self.packed_indirect_buffer,
            0,
            packed_indirect_args.as_bytes(),
        );
        renderer.write_buffer(
            &self.aabb_visible_buffer,
            0,
            bytemuck::cast_slice(&self.aabb_visible_batch),
        );
        // update mdi args and meshing queue
        compute_pass.set_pipeline(&self.culled_mdi_args_pipeline);
        compute_pass.set_bind_group(0, &self.culled_mdi_args_bind_group, &[]);
        let batch_size = self.aabb_visible_batch.len() as u32;
        compute_pass.set_push_constants(0, bytemuck::bytes_of(&batch_size));
        let wg_count = dispatch_count_1d(batch_size, self.config.max_workgroup_size_1d);
        compute_pass.dispatch_workgroups(wg_count, 1, 1);

        // handle meshing queue
        compute_pass.set_pipeline(&self.meshing_pipeline);
        compute_pass.set_bind_group(0, &self.meshing_bind_group, &[]);
        compute_pass.dispatch_workgroups_indirect(&self.packed_indirect_buffer, 4 * 4);
    }

    pub fn drop_chunk(&mut self, position: &IVec3) {
        let (_, mesh_state) = self.gpu_cached.remove(position).unwrap();
        if let ChunkMeshState::Meshed(mesh_entry) = mesh_state {
            self.gpu_mesh_allocator
                .deallocate(mesh_entry.face_alloc)
                .unwrap();
        }
    }

    pub fn draw(&mut self, renderer: &Renderer<'_>, render_pass: &mut RenderPass) {
        if self.aabb_visible_batch.is_empty() {
            return;
        }
        render_pass.set_bind_group(1, &self.render_bind_group, &[]);
        render_pass.multi_draw_indirect_count(
            &renderer.indirect_buffer,
            0,
            &self.packed_indirect_buffer,
            0,
            self.config.max_indirect_count,
        );
    }
}

fn dispatch_count_1d(tasks: u32, workgroup_size: u32) -> u32 {
    (tasks as f32 / workgroup_size as f32).ceil() as u32
}

fn chunk_render_bind_group(
    device: &Device,
    camera_buffer: &VxBuffer,
    face_data_buffer: &VxBuffer,
) -> (BindGroupLayout, BindGroup) {
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Chunk Render Bind Group Layout"),
        entries: &[
            camera_buffer.bind_layout_entry(0, false, ShaderStages::VERTEX),
            face_data_buffer.bind_layout_entry(1, true, ShaderStages::VERTEX),
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Chunk Render Bind Group"),
        layout: &layout,
        entries: &resources::utils::bind_entries([
            camera_buffer.as_entire_binding(),
            face_data_buffer.as_entire_binding(),
        ]),
    });
    (layout, bind_group)
}

fn chunk_meshing_bgl(
    device: &Device,
    chunk_buffer_size: BufferSize,
    face_data_buffer_size: BufferSize,
    mesh_queue_buffer_size: BufferSize,
) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Chunk Compute Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0, // chunk data
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(chunk_buffer_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1, // face data
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(face_data_buffer_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2, // mesh queue
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(mesh_queue_buffer_size),
                },
                count: None,
            },
        ],
    })
}

fn chunk_write_bgl(device: &Device, dst_size: BufferSize, src_size: BufferSize) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Chunk Write Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0, // write dst
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(dst_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1, // write src
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(src_size),
                },
                count: None,
            },
        ],
    })
}

fn culled_mdi_args_bgl(
    device: &Device,
    indirect_size: BufferSize,
    packed_indirect_size: BufferSize,
    meshing_batch_size: BufferSize,
    chunks_size: BufferSize,
    chunks_in_view_size: BufferSize,
    camera_size: BufferSize,
) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Chunk Draw Args Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0, // indirect
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(indirect_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1, // meshing indirect
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(packed_indirect_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2, // meshing batch
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(meshing_batch_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3, // chunks
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(chunks_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4, // chunks in view
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(chunks_in_view_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 5, // camera
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(camera_size),
                },
                count: None,
            },
        ],
    })
}

fn culled_mdi_args_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create_shader(
        device,
        resources::shader::chunk_culled_mdi_args_wgsl().into(),
        "Chunk Culled MDI Args Pipeline Shader",
    );
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Chunk Culled MDI Args Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStages::COMPUTE,
            range: 0..4,
        }],
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Chunk Culled MDI Args Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("write_culled_mdi"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn chunk_meshing_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create_shader(
        device,
        resources::shader::chunk_meshing_wgsl().into(),
        "Chunk Meshing Pipeline Shader",
    );
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Chunk Meshing Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Chunk Meshing Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("mesh_chunks_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn chunk_write_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create_shader(
        device,
        resources::shader::chunk_write_wgsl().into(),
        "Chunk Write Pipeline Shader",
    );
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Chunk Write Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStages::COMPUTE,
            range: 0..4,
        }],
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Chunk Write Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("chunk_write_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}
