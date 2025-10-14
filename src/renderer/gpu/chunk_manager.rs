use crate::compute::MIB;
use crate::compute::geo::{Frustum, Plane, chunk_to_world_pos};
use crate::renderer::gpu::chunk_entry::{GPU4Bytes, GPUChunkMeshEntry};
use crate::renderer::gpu::{GPUVoxelChunk, GPUVoxelChunkAdjContent, GPUVoxelFaceData};
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::{Renderer, VxDrawIndirectBatch, resources};
use crate::world::types::{CHUNK_DIM, Chunk};
use glam::IVec3;
use rustc_hash::FxHashMap;
use slabmap::SlabMap;
use suballoc::SubAllocator;
use wgpu::wgt::DrawIndirectArgs;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferSize, BufferUsages, CommandEncoder,
    ComputePass, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device,
    PipelineLayoutDescriptor, RenderPass, ShaderStages,
};

type BufferDrawArgs = FxHashMap<usize, DrawIndirectArgs>;

#[derive(Debug, Clone)]
pub enum ChunkMeshState {
    Meshed { face_count: u32, allocation: u32 },
    Unmeshed { face_count: u32 },
}

pub struct ChunkManager {
    gpu_chunks_mesh_allocator: SubAllocator,
    pub gpu_chunk_cache: SlabMap<IVec3, ChunkMeshState>, //fixme temp

    gpu_active_draw: BufferDrawArgs,
    gpu_chunk_writes: Vec<GPUVoxelChunk>,
    gpu_chunk_meshing_queue: Vec<GPUChunkMeshEntry>,
    gpu_chunk_in_view: Vec<GPUChunkMeshEntry>,

    max_write_count: usize,
    render_max_sq: i32,

    draw_args_pipeline: ComputePipeline,
    draw_args_bind_group: BindGroup,

    chunk_buffer: VxBuffer,
    aabb_visible_chunk_buffer: VxBuffer,
    chunk_write_buffer: VxBuffer,
    mesh_queue_buffer: VxBuffer,
    faces_buffer: VxBuffer,
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
        render_distance: i32,
        camera_buffer: &VxBuffer,
        max_face_count: usize,
        max_chunk_count: usize,
        max_chunk_write_count: usize,
    ) -> Self {
        let voxel_chunk_buffer = renderer.device.create_vx_buffer::<GPUVoxelChunk>(
            "Voxel Chunks Buffer",
            max_chunk_count,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let voxel_chunk_write_buffer = renderer.device.create_vx_buffer::<GPUVoxelChunk>(
            "Voxel Chunk Write Buffer",
            max_chunk_write_count,
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        );

        let voxel_face_buffer = renderer.device.create_vx_buffer::<GPUVoxelFaceData>(
            "Voxel Face Data Buffer",
            max_face_count,
            BufferUsages::VERTEX | BufferUsages::STORAGE,
        );

        let voxel_mesh_queue_buffer = renderer.device.create_vx_buffer::<GPUChunkMeshEntry>(
            "Voxel Mesh Queue Buffer",
            max_chunk_count, // fixme overkill but cheap anyway?
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let aabb_visible_chunk_buffer = renderer.device.create_vx_buffer::<GPUChunkMeshEntry>(
            "AABB Visible Voxel Chunk Buffer",
            max_chunk_count, // fixme overkill but cheap anyway?
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let draw_args_bgl = chunk_draw_args_bgl(
            &renderer.device,
            renderer.indirect_buffer.buffer_size,
            renderer.indirect_count_buffer.buffer_size,
            voxel_chunk_buffer.buffer_size,
            aabb_visible_chunk_buffer.buffer_size,
            camera_buffer.buffer_size,
        );
        let draw_args_pipeline = chunk_draw_args_pipeline(&renderer.device, &[&draw_args_bgl]);
        let draw_args_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Draw Args Bind Group"),
            layout: &draw_args_bgl,
            entries: &resources::utils::bind_entries([
                renderer.indirect_buffer.as_entire_binding(),
                renderer.indirect_count_buffer.as_entire_binding(),
                voxel_chunk_buffer.as_entire_binding(),
                aabb_visible_chunk_buffer.as_entire_binding(),
                camera_buffer.as_entire_binding(),
            ]),
        });

        let write_bgl = chunk_write_bgl(
            &renderer.device,
            voxel_chunk_buffer.buffer_size,
            voxel_chunk_write_buffer.buffer_size,
        );
        let write_pipeline = chunk_write_pipeline(&renderer.device, &[&write_bgl]);
        let write_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Write Bind Group"),
            layout: &write_bgl,
            entries: &resources::utils::bind_entries([
                voxel_chunk_buffer.as_entire_binding(),
                voxel_chunk_write_buffer.as_entire_binding(),
            ]),
        });

        let meshing_bgl = chunk_meshing_bgl(
            &renderer.device,
            voxel_chunk_buffer.buffer_size,
            voxel_face_buffer.buffer_size,
            voxel_mesh_queue_buffer.buffer_size,
        );
        let meshing_pipeline = chunk_meshing_pipeline(&renderer.device, &[&meshing_bgl]);
        let meshing_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Meshing Bind Group"),
            layout: &meshing_bgl,
            entries: &resources::utils::bind_entries([
                voxel_chunk_buffer.as_entire_binding(),
                voxel_face_buffer.as_entire_binding(),
                voxel_mesh_queue_buffer.as_entire_binding(),
            ]),
        });

        let (render_bind_group_layout, render_bind_group) =
            chunk_render_bind_group(&renderer.device, camera_buffer, &voxel_face_buffer);

        Self {
            gpu_chunks_mesh_allocator: SubAllocator::new(max_face_count as u32),
            gpu_chunk_cache: SlabMap::with_capacity(max_chunk_count),
            gpu_active_draw: FxHashMap::default(),
            gpu_chunk_writes: Vec::with_capacity(max_chunk_count),
            chunk_buffer: voxel_chunk_buffer,
            chunk_write_buffer: voxel_chunk_write_buffer,
            faces_buffer: voxel_face_buffer,
            max_write_count: max_chunk_write_count,
            mesh_queue_buffer: voxel_mesh_queue_buffer,
            gpu_chunk_meshing_queue: Vec::with_capacity(max_chunk_count), // fixme change this if/when changing buffer size
            gpu_chunk_in_view: Vec::with_capacity(1024), // fixme arbitrary number and insufficient
            render_max_sq: render_distance.pow(2),
            draw_args_pipeline,
            draw_args_bind_group,
            aabb_visible_chunk_buffer,
            write_pipeline,
            write_bind_group,
            meshing_pipeline,
            meshing_bind_group,
            render_bind_group,
            render_bind_group_layout,
        }
    }

    fn insert_chunk(&mut self, chunk: &Chunk) -> GPUVoxelChunk {
        let face_count = chunk.face_count.unwrap() as u32;
        let render_meta = ChunkMeshState::Unmeshed { face_count };
        let chunk_index = self.gpu_chunk_cache.insert(chunk.position, render_meta);
        let position_index = chunk.position.extend(chunk_index as i32);

        // fixme avoid copying until transmutation?
        GPUVoxelChunk::new(position_index, &chunk.adjacent_blocks, &chunk.blocks)
    }

    pub fn update_gpu_chunk_writes(&mut self, chunks: &[Chunk]) {
        debug_assert!(chunks.len() <= self.max_write_count);
        self.gpu_chunk_writes.clear();
        unsafe { self.gpu_chunk_writes.set_len(1) }; // index 0 is reserved for write_count

        for chunk in chunks {
            if self.is_chunk_cached(&chunk.position) {
                self.drop_chunk(&chunk.position);
            }
            let gpu_chunk = self.insert_chunk(chunk);
            self.gpu_chunk_writes.push(gpu_chunk);
        }
        let write_count = self.gpu_chunk_writes.len() as u32 - 1;
        self.gpu_chunk_writes[0].set_chunk_index(write_count);
    }

    pub fn encode_gpu_chunk_writes(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        let write_count = self.gpu_chunk_writes.len() as u32 - 1; // 1 is reserved for write_count
        if write_count == 0 {
            return;
        }
        renderer.write_buffer(
            &self.chunk_buffer,
            0,
            bytemuck::cast_slice(&self.gpu_chunk_writes),
        );
        compute_pass.set_pipeline(&self.write_pipeline);
        compute_pass.set_bind_group(0, &self.write_bind_group, &[]);
        // fixme unify in a config
        let workgroup_count = (write_count as f32 / 16f32.powf(2.0)).ceil() as u32;
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
    }

    fn should_chunk_render(&self, origin: IVec3, ch_pos: IVec3, view_planes: &[Plane; 6]) -> bool {
        let min = chunk_to_world_pos(ch_pos);
        Frustum::aabb_within_frustum(min, min + CHUNK_DIM as f32, view_planes)
            && origin.distance_squared(ch_pos) < self.render_max_sq
    }

    pub fn update_gpu_view_chunks(
        &mut self,
        camera_ch_position: IVec3,
        view_planes: &[Plane; 6],
        mut missing_chunk: impl FnMut(IVec3),
    ) {
        let mut frustum_aabb = Frustum::aabb(view_planes);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();

        self.gpu_chunk_meshing_queue.clear();
        self.gpu_chunk_in_view.clear();
        unsafe { self.gpu_chunk_in_view.set_len(1) };

        for (_, slab_idx, render_mesh_state) in self.gpu_chunk_cache.iter_mut() {
            match render_mesh_state {
                ChunkMeshState::Meshed {
                    face_count,
                    allocation,
                } => {
                    let mesh_entry =
                        GPUChunkMeshEntry::new(slab_idx as u32, *face_count, *allocation);
                    self.gpu_chunk_in_view.push(mesh_entry);
                }
                ChunkMeshState::Unmeshed { face_count } if *face_count != 0 => {
                    let fc = *face_count;
                    let allocation = self.gpu_chunks_mesh_allocator.allocate(fc).unwrap();
                    *render_mesh_state = ChunkMeshState::Meshed {
                        face_count: fc,
                        allocation,
                    };
                    let mesh_entry = GPUChunkMeshEntry::new(slab_idx as u32, fc, allocation);
                    println!("{} at {}, fc: {}", allocation, slab_idx, fc);
                    self.gpu_chunk_meshing_queue.push(mesh_entry);
                    self.gpu_chunk_in_view.push(mesh_entry);
                }
                _ => (),
            }
        }
        frustum_aabb.discrete_points(|ch_pos| {
            // if !self.should_chunk_render(camera_ch_position, ch_pos, view_planes) {
            //     return;
            // }
            if let None = self.gpu_chunk_cache.get(&ch_pos) {
                missing_chunk(ch_pos);
            }
        });
        // frustum_aabb.discrete_points(|ch_pos| {
        //     // fixme better enum here
        //     match self.gpu_chunk_cache.get_mut(&ch_pos) {
        //         Some((slab_idx, render_mesh_state)) => match render_mesh_state {
        //             ChunkMeshState::Meshed {
        //                 face_count,
        //                 allocation,
        //             } if *face_count != 0 => {
        //                 let mesh_entry =
        //                     GPUChunkMeshEntry::new(slab_idx as u32, *face_count, *allocation);
        //                 self.gpu_chunk_in_view.push(mesh_entry);
        //             }
        //             ChunkMeshState::Unmeshed { face_count } if *face_count != 0 => {
        //                 let fc = *face_count;
        //                 let allocation = self.gpu_chunks_mesh_allocator.allocate(fc).unwrap();
        //                 *render_mesh_state = ChunkMeshState::Meshed {
        //                     face_count: fc,
        //                     allocation,
        //                 };
        //                 let mesh_entry = GPUChunkMeshEntry::new(slab_idx as u32, fc, allocation);
        //                 self.gpu_chunk_meshing_queue.push(mesh_entry);
        //                 self.gpu_chunk_in_view.push(mesh_entry);
        //             }
        //             _ => (),
        //         },
        //         None => missing_chunk(ch_pos),
        //     };
        // });

        // println!("updating view chunks: {}", self.gpu_chunk_in_view.len());

        self.gpu_chunk_in_view[0].index = self.gpu_chunk_in_view.len() as u32 - 1;
    }

    pub fn encode_gpu_view_chunks(
        &mut self,
        renderer: &Renderer<'_>,
        compute_pass: &mut ComputePass,
    ) {
        // fixme temp
        renderer.write_buffer(
            &renderer.indirect_count_buffer,
            0,
            bytemuck::cast_slice(&[0u32]),
        );

        // index 0 reserved
        if !self.gpu_chunk_in_view.len() > 1 {
            renderer.write_buffer(
                &self.aabb_visible_chunk_buffer,
                0,
                bytemuck::cast_slice(&self.gpu_chunk_in_view),
            );
            compute_pass.set_pipeline(&self.draw_args_pipeline);
            compute_pass.set_bind_group(0, &self.draw_args_bind_group, &[]);
            let wg_count = ((self.gpu_chunk_in_view.len() - 1) as f32 / 16u32.pow(2) as f32).ceil();
            compute_pass.dispatch_workgroups(wg_count as u32, 1, 1);
        }

        if !self.gpu_chunk_meshing_queue.is_empty() {
            println!("{:?}", self.gpu_chunk_meshing_queue);
            renderer.write_buffer(
                &self.mesh_queue_buffer,
                0,
                bytemuck::cast_slice(&self.gpu_chunk_meshing_queue),
            );
            compute_pass.set_pipeline(&self.meshing_pipeline);
            compute_pass.set_bind_group(0, &self.meshing_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.gpu_chunk_meshing_queue.len() as u32, 1, 1);
        }
    }

    pub fn drop_chunk(&mut self, position: &IVec3) {
        let (_, mesh_state) = self.gpu_chunk_cache.remove(position).unwrap();
        if let ChunkMeshState::Meshed { allocation, .. } = mesh_state {
            self.gpu_chunks_mesh_allocator
                .deallocate(allocation)
                .unwrap();
        }
    }
    //
    // pub fn retain_chunk_positions<F: FnMut(&IVec3) -> bool>(&mut self, mut func: F) {
    //     let to_drop = self
    //         .gpu_chunk_cache
    //         .iter()
    //         .filter_map(|(p, _)| (!func(p)).then_some(p).cloned())
    //         .collect::<Vec<_>>();
    //     for p in to_drop {
    //         self.drop_chunk(p);
    //     }
    // }

    pub fn draw(&mut self, renderer: &Renderer<'_>, render_pass: &mut RenderPass) {
        // fixme handle no chunks to render..
        render_pass.set_bind_group(1, &self.render_bind_group, &[]);
        render_pass.multi_draw_indirect_count(
            &renderer.indirect_buffer,
            0,
            &renderer.indirect_count_buffer,
            0,
            32000, // fixme arbitrary number
        );
        // render_pass.multi_draw_indirect(&renderer.indirect_buffer, 0, render_count);
    }

    pub fn is_chunk_cached(&self, position: &IVec3) -> bool {
        self.gpu_chunk_cache.get(position).is_some()
    }
    //
    // fn allocate_chunk_mesh(&mut self, chunk: &Chunk) -> GPUVoxelChunk {
    //     let face_count = chunk.face_count.unwrap() as u32;
    //     let render_meta = ChunkRenderMeta::default();
    //     let slab_index = self.gpu_chunks.insert(chunk.position, render_meta);
    //     let header = GPUVoxelChunkHeader::new(
    //         render_meta.mesh_allocation,
    //         face_count,
    //         slab_index as u32,
    //         chunk.position,
    //     );
    //
    //     let adjacent_blocks: GPUVoxelChunkAdjContent =
    //         unsafe { std::mem::transmute(chunk.adjacent_blocks) };
    //
    //     GPUVoxelChunk::new(header, adjacent_blocks, chunk.blocks)
    // }

    // fn write_indirect_draw_args(&self, renderer: &Renderer<'_>, buffer_draw_args: &BufferDrawArgs) {
    //     // todo encode batch on first iter?
    //     let draw_indirect_batch = VxDrawIndirectBatch::from_iter(buffer_draw_args.values());
    //     renderer.write_buffer(
    //         &renderer.indirect_buffer,
    //         0,
    //         bytemuck::cast_slice(&draw_indirect_batch.encode(renderer.adapter_info().backend)),
    //     );
    // }

    pub fn mem_debug_throttled(&self) {
        use crate::call_every;

        // the blob clears cli
        let capacity = self.gpu_chunks_mesh_allocator.capacity();
        let free_percent = (self.gpu_chunks_mesh_allocator.free() as f32 / capacity as f32) * 100.0;
        call_every!(ALLOC_DBG, 50, || println!(
            "\x1B[2J\x1B[1;1Hfree: {:>3.1}% capacity: {}",
            free_percent, capacity,
        ));
    }
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

fn chunk_draw_args_bgl(
    device: &Device,
    indirect_size: BufferSize,
    indirect_count_size: BufferSize,
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
                binding: 1, // indirect count
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(indirect_count_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2, // chunks
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(chunks_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3, // chunks in view
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(chunks_in_view_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4, // camera
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

fn chunk_draw_args_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create(
        device,
        resources::shader::chunk_draw_args().into(),
        "chunk draw args pipeline shader",
    );
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Chunk Draw Args Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Chunk Draw Args Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("write_chunk_indirect_draw_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn chunk_meshing_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create(
        device,
        resources::shader::chunk_meshing().into(),
        "chunk meshing pipeline shader",
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
    let shader = resources::shader::create(
        device,
        resources::shader::chunk_write().into(),
        "chunk write pipeline shader",
    );
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Chunk Write Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
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
