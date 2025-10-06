use crate::call_every;
use crate::compute::ds::Slap;
use crate::renderer::gpu::chunk_entry::GPUVoxelChunkHeader;
use crate::renderer::gpu::{GPUVoxelChunk, GPUVoxelChunkAdjContent, GPUVoxelFaceData};
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::{Renderer, VxDrawIndirectBatch, resources};
use crate::world::types::Chunk;
use glam::IVec3;
use std::collections::HashMap;
use std::num::NonZeroU64;
use rustc_hash::FxHashMap;
use suballoc::SubAllocator;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferSize, BufferUsages, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, RenderPass, ShaderStages};
use wgpu::wgt::DrawIndirectArgs;

type BufferDrawArgs = FxHashMap<usize, DrawIndirectArgs>;

pub struct ChunkManager {
    suballocator: SubAllocator,
    suballocs: Slap<IVec3, u32>,

    gpu_active_draw: BufferDrawArgs,
    gpu_chunk_writes: Vec<GPUVoxelChunk>,

    chunks_buffer: VxBuffer,
    faces_buffer: VxBuffer,
    meshing_pipeline: ComputePipeline,
    meshing_bind_group: BindGroup,
    render_bind_group: BindGroup,
    pub(crate) render_bind_group_layout: BindGroupLayout,
}

impl ChunkManager {
    pub fn new(
        renderer: &Renderer<'_>,
        view_projection_buffer: &VxBuffer,
        max_face_count: usize,
        max_chunk_count: usize,
    ) -> Self {
        let voxel_chunk_buffer = renderer.device.create_vx_buffer::<GPUVoxelChunk>(
            "Voxel Chunks Buffer",
            max_chunk_count,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let voxel_face_buffer = renderer.device.create_vx_buffer::<GPUVoxelFaceData>(
            "Voxel Face Data Buffer",
            max_face_count,
            BufferUsages::VERTEX | BufferUsages::STORAGE,
        );

        let chunk_buffer_size = NonZeroU64::new(voxel_chunk_buffer.size()).unwrap();
        let face_buffer_size = NonZeroU64::new(voxel_face_buffer.size()).unwrap();

        let meshing_bgl = chunk_meshing_bgl(&renderer.device, chunk_buffer_size, face_buffer_size);
        let meshing_pipeline = chunk_meshing_pipeline(&renderer.device, &[&meshing_bgl]);
        let meshing_bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk Meshing Bind Group"),
            layout: &meshing_bgl,
            entries: &resources::utils::bind_entries([
                voxel_chunk_buffer.as_entire_binding(),
                voxel_face_buffer.as_entire_binding(),
            ]),
        });

        let (render_bind_group_layout, render_bind_group) =
            chunk_render_bind_group(&renderer.device, view_projection_buffer, &voxel_face_buffer);

        Self {
            suballocator: SubAllocator::new(max_face_count as u32),
            suballocs: Slap::new(),
            gpu_active_draw: FxHashMap::default(),
            gpu_chunk_writes: Vec::with_capacity(max_chunk_count),
            chunks_buffer: voxel_chunk_buffer,
            faces_buffer: voxel_face_buffer,
            meshing_pipeline,
            meshing_bind_group,
            render_bind_group,
            render_bind_group_layout,
        }
    }

    pub fn encode_new_chunks(
        &mut self,
        renderer: &Renderer<'_>,
        encoder: &mut CommandEncoder,
        chunks: &[Chunk],
    ) {
        self.gpu_chunk_writes.clear();
        for chunk in chunks {
            if chunk.face_count.unwrap() == 0 {
                continue;
            }
            if self.is_rendered(chunk.position) {
                // needs to be remeshed, dropping existing one first
                self.drop_chunk_position(chunk.position);
            }
            let gpu_chunk = self.allocate_chunk(chunk);
            self.gpu_chunk_writes.push(gpu_chunk);
        }
        self.encode_meshing_pass(renderer, encoder);
    }

    pub fn drop_chunk_position(&mut self, position: IVec3) {
        let slap_entry_opt = self.suballocs.remove(&position);
        let (slab_index, alloc_start) = slap_entry_opt.unwrap();
        self.gpu_active_draw.remove(&slab_index).unwrap();
        self.suballocator.deallocate(alloc_start).unwrap();
    }

    pub fn retain_chunk_positions<F: FnMut(&IVec3) -> bool>(&mut self, mut func: F) {
        let to_drop = self
            .suballocs
            .iter()
            .filter_map(|(p, _)| (!func(p)).then_some(p).cloned())
            .collect::<Vec<_>>();
        for p in to_drop {
            self.drop_chunk_position(p);
        }
    }

    pub fn draw(&mut self, renderer: &Renderer<'_>, render_pass: &mut RenderPass) {
        self.write_indirect_draw_args(renderer, &self.gpu_active_draw);
        let render_count = self.gpu_active_draw.len() as u32;
        if render_count != 0 {
            render_pass.set_bind_group(1, &self.render_bind_group, &[]);
            render_pass.multi_draw_indirect(&renderer.indirect_buffer, 0, render_count);
        }
    }

    pub fn is_rendered(&self, position: IVec3) -> bool {
        self.suballocs.contains(&position)
    }

    fn allocate_chunk(&mut self, chunk: &Chunk) -> GPUVoxelChunk {
        let face_count = chunk.face_count.unwrap() as u32;
        let mesh_alloc = self.suballocator.allocate(face_count).unwrap();
        let slab_index = self.suballocs.insert(chunk.position, mesh_alloc);
        let header = GPUVoxelChunkHeader::new(
            mesh_alloc as u32,
            face_count,
            slab_index as u32,
            chunk.position,
        );

        let adjacent_blocks: GPUVoxelChunkAdjContent = unsafe {
            std::mem::transmute(chunk.adjacent_blocks)
        };

        GPUVoxelChunk::new(header, adjacent_blocks, chunk.blocks)
    }

    fn encode_meshing_pass(
        &mut self,
        renderer: &Renderer<'_>,
        encoder: &mut CommandEncoder,
    ) {
        if self.gpu_chunk_writes.is_empty() {
            return;
        }
        renderer.write_buffer(&self.chunks_buffer, 0, bytemuck::cast_slice(&self.gpu_chunk_writes));
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Chunk Meshing Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.meshing_pipeline);
            compute_pass.set_bind_group(0, &self.meshing_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.gpu_chunk_writes.len() as u32, 1, 1);
        }
        for entry in self.gpu_chunk_writes.iter() {
            self.gpu_active_draw.insert(
                entry.header.slab_index as usize,
                entry.header.draw_indirect_args(),
            );
        }
    }

    fn write_indirect_draw_args(&self, renderer: &Renderer<'_>, buffer_draw_args: &BufferDrawArgs) {
        // todo encode batch on first iter?
        let draw_indirect_batch = VxDrawIndirectBatch::from_iter(buffer_draw_args.values());
        renderer.write_buffer(
            &renderer.indirect_buffer,
            0,
            bytemuck::cast_slice(&draw_indirect_batch.encode(renderer.adapter_info().backend)),
        );
    }

    pub fn mem_debug_throttled(&self) {
        // the blob clears cli
        let capacity = self.suballocator.capacity();
        let free_percent = (self.suballocator.free() as f32 / capacity as f32) * 100.0;
        call_every!(ALLOC_DBG, 50, || println!(
            "\x1B[2J\x1B[1;1Hfree: {:>3.1}% capacity: {}",
            free_percent, capacity,
        ));
    }
}

fn chunk_render_bind_group(
    device: &Device,
    view_projection_buffer: &VxBuffer,
    face_data_buffer: &VxBuffer,
) -> (BindGroupLayout, BindGroup) {
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Chunk Render Bind Group Layout"),
        entries: &[
            view_projection_buffer.bind_layout_entry(0, false, ShaderStages::VERTEX),
            face_data_buffer.bind_layout_entry(1, true, ShaderStages::VERTEX),
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Chunk Render Bind Group"),
        layout: &layout,
        entries: &resources::utils::bind_entries([
            view_projection_buffer.as_entire_binding(),
            face_data_buffer.as_entire_binding(),
        ]),
    });
    (layout, bind_group)
}

pub fn chunk_meshing_bgl(
    device: &Device,
    chunk_buffer_size: BufferSize,
    face_data_buffer_size: BufferSize,
) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Chunk Compute Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0, // chunk entry data
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(chunk_buffer_size),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1, // face data buffer
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(face_data_buffer_size),
                },
                count: None,
            },
        ],
    })
}

pub fn chunk_meshing_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create(device, resources::shader::chunk_meshing().into());
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
