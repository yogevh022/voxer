use super::chunk_render::ChunkRender;
use crate::renderer::gpu::GPUVoxelChunk;
use crate::renderer::gpu::chunk_manager::BufferDrawArgs;
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::resources::vx_device::VxDevice;
use crate::renderer::{Renderer, resources};
use std::num::NonZeroU64;
use wgpu::{BufferAddress, BufferUsages};

pub struct ChunkCompute {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    chunks_buffer: VxBuffer,
    staging_chunks_buffer: VxBuffer,
}

impl ChunkCompute {
    pub fn init(device: &VxDevice, chunk_render: &ChunkRender, max_chunk_count: usize) -> Self {
        let min_chunk =
            NonZeroU64::new((max_chunk_count * size_of::<GPUVoxelChunk>()) as u64).unwrap();
        let min_face_data = NonZeroU64::new(chunk_render.face_data_buffer.size()).unwrap();

        let chunk_layout = chunk_bind_group_layout(device, min_chunk, min_face_data);
        let pipeline = create_chunk_compute_pipeline(device, &[&chunk_layout]);

        let chunks_buffer = device.create_vx_buffer::<GPUVoxelChunk>(
            "Chunks Buffer",
            max_chunk_count.try_into().unwrap(),
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let staging_chunks_buffer = device.create_vx_buffer::<GPUVoxelChunk>(
            "Staging Chunks Buffer",
            max_chunk_count.try_into().unwrap(), // fixme temp
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Chunk Compute Bind Group"),
            layout: &chunk_layout,
            entries: &resources::utils::bind_entries([
                chunks_buffer.as_entire_binding(),
                chunk_render.face_data_buffer.as_entire_binding(),
                staging_chunks_buffer.as_entire_binding(),
            ]),
        });

        Self {
            pipeline,
            bind_group,
            chunks_buffer,
            staging_chunks_buffer,
        }
    }

    pub fn write_chunks(&self, renderer: &Renderer<'_>, buffer_writes: &[GPUVoxelChunk]) {
        if buffer_writes.is_empty() {
            return;
        }
        renderer.write_buffer(
            &self.staging_chunks_buffer,
            0,
            bytemuck::cast_slice(&[buffer_writes.len() as i32]), // fixme temp
        );
        renderer.write_buffer(
            &self.staging_chunks_buffer,
            self.staging_chunks_buffer.stride() as BufferAddress,
            bytemuck::cast_slice(buffer_writes),
        );

        let staging_workgroup_size: u32 = 256; // fixme this needs to be synced with gpu shader!
        let mut encoder = renderer.create_encoder("Chunk Staging Compute Encoder");
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Chunk Staging Compute Pass"),
                timestamp_writes: None,
            });
            let workgroup_count = 1 + (buffer_writes.len() - 1) as u32 / staging_workgroup_size;
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }
        renderer.queue.submit(Some(encoder.finish())); // fixme can we coalesce submits?
    }

    pub fn dispatch_meshing_workgroups(
        &mut self,
        renderer: &Renderer<'_>,
        active_draw: &mut BufferDrawArgs,
        buffer_writes: Vec<GPUVoxelChunk>,
    ) {
        let mut encoder = renderer.create_encoder("Chunk Meshing Compute Encoder");
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Chunk Meshing Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(buffer_writes.len() as u32, 1, 1);
        }
        for entry in buffer_writes {
            active_draw.insert(
                entry.header.slab_index as usize,
                entry.header.draw_indirect_args(),
            );
        }
        renderer.queue.submit(Some(encoder.finish()));
    }
}

pub fn chunk_bind_group_layout(
    device: &wgpu::Device,
    chunk_buffer_size: wgpu::BufferSize,
    face_data_buffer_size: wgpu::BufferSize,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Chunk Compute Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0, // chunks data
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(chunk_buffer_size),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1, // face data buffer
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(face_data_buffer_size),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2, // staging chunks data
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(chunk_buffer_size),
                },
                count: None,
            },
        ],
    })
}

pub fn create_chunk_compute_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    let shader = resources::shader::create(device, resources::shader::chunk_meshing().into());
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Chunk Compute Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Chunk Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("mesh_chunks_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}
