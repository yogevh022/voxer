use super::chunk_render::ChunkRender;
use crate::const_labels;
use crate::renderer::gpu::GPUChunkEntry;
use crate::renderer::gpu::chunk_manager::{BufferDrawArgs};
use crate::renderer::{
    DrawIndexedIndirectArgsA32, Renderer, RendererBuilder, resources,
};
use std::array;
use std::num::NonZeroU64;

pub struct ChunkCompute<const N_BUFF: usize> {
    pipeline: wgpu::ComputePipeline,
    bind_groups: [wgpu::BindGroup; N_BUFF],
    chunk_buffers: [wgpu::Buffer; N_BUFF],
}

impl<const N_BUFF: usize> ChunkCompute<N_BUFF> {
    const CHUNK_BUFFER_LABELS: [&'static str; N_BUFF] =
        const_labels!("chunk", N_BUFF);

    pub fn init(
        device: &wgpu::Device,
        chunk_render: &ChunkRender<N_BUFF>,
        chunks_buffer_size: wgpu::BufferAddress,
    ) -> Self {
        let min_vertex = NonZeroU64::new(chunk_render.vertex_buffers[0].size()).unwrap();
        let min_index = NonZeroU64::new(chunk_render.index_buffers[0].size()).unwrap();
        let min_mmat = NonZeroU64::new(chunk_render.mmat_buffer.size()).unwrap();
        let min_chunk = NonZeroU64::new(chunks_buffer_size).unwrap();

        let layout = chunk_bind_group_layout(device, min_chunk, min_vertex, min_index, min_mmat);
        let pipeline = create_chunk_compute_pipeline(device, &[&layout]);

        let chunk_buffers = array::from_fn(|i| {
            staging_chunk_init(device, Self::CHUNK_BUFFER_LABELS[i], chunks_buffer_size)
        });
        let bind_groups = array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(("chunk_compute_bind_group_".to_owned() + &*i.to_string()).as_str()),
                layout: &layout,
                entries: &resources::utils::index_based_entries([
                    chunk_buffers[i].as_entire_binding(),
                    chunk_render.vertex_buffers[i].as_entire_binding(),
                    chunk_render.index_buffers[i].as_entire_binding(),
                    chunk_render.mmat_buffer.as_entire_binding(),
                ]),
            })
        });
        Self {
            pipeline,
            bind_groups,
            chunk_buffers,
        }
    }

    pub fn write_chunks(
        &self,
        renderer: &Renderer<'_>,
        buffer_writes: &[Vec<GPUChunkEntry>; N_BUFF],
    ) {
        for i in 0..N_BUFF {
            if buffer_writes[i].is_empty() {
                continue;
            }
            renderer.write_buffer(
                &self.chunk_buffers[i],
                0u64,
                bytemuck::cast_slice(&buffer_writes[i]),
            );
        }
    }

    pub fn dispatch_staging_workgroups(
        &mut self,
        renderer: &Renderer<'_>,
        active_draw: &mut BufferDrawArgs<N_BUFF>,
        buffer_writes: [Vec<GPUChunkEntry>; N_BUFF],
    ) {
        let mut encoder = renderer.create_encoder("chunk_mesh_compute_encoder");
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("chunk_meshing_compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            for (buffer_i, entries) in buffer_writes.iter().enumerate() {
                compute_pass.set_bind_group(0, &self.bind_groups[buffer_i], &[]);
                compute_pass.dispatch_workgroups(entries.len() as u32, 1, 1);
            }
        }
        for (buffer_i, entries) in buffer_writes.into_iter().enumerate() {
            for entry in entries {
                active_draw[buffer_i].insert(
                    entry.header.slab_index as usize,
                    entry.header.draw_indexed_indirect_args()
                );
            }
        }
        renderer.queue.submit(Some(encoder.finish()));
    }
}

pub fn chunk_bind_group_layout(
    device: &wgpu::Device,
    min_chunk: wgpu::BufferSize,
    min_vertex: wgpu::BufferSize,
    min_index: wgpu::BufferSize,
    min_mmat: wgpu::BufferSize,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("chunk_compute_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0, // chunk block data
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(min_chunk),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1, // vertex buffer (bound as storage)
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(min_vertex),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2, // index buffer (bound as storage)
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(min_index),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3, // model mat buffer
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(min_mmat),
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
        label: Some("chunk_compute_pipeline_layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("chunk_compute_pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("mesh_chunks_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn staging_chunk_init(
    device: &wgpu::Device,
    label: &str,
    size: wgpu::BufferAddress,
) -> wgpu::Buffer {
    RendererBuilder::make_buffer(
        &device,
        label,
        size,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
    )
}
