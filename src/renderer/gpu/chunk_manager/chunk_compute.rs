use crate::const_labels;
use crate::renderer::gpu::GPUChunkEntry;
use crate::renderer::{Renderer, RendererBuilder, resources, Vertex, Index};
use std::array;
use glam::Mat4;
use super::chunk_render::ChunkRender;

pub struct ChunkCompute<const NumStagingBuffers: usize> {
    pipeline: wgpu::ComputePipeline,
    bind_groups: [wgpu::BindGroup; NumStagingBuffers],
    staging_chunk_buffers: [wgpu::Buffer; NumStagingBuffers],
    staging_vertex_buffers: [wgpu::Buffer; NumStagingBuffers],
    staging_index_buffers: [wgpu::Buffer; NumStagingBuffers],
    staging_mmat_buffers: [wgpu::Buffer; NumStagingBuffers],
}

impl<const NumStagingBuffers: usize> ChunkCompute<NumStagingBuffers> {
    const STAGING_CHUNK_LABELS: [&'static str; NumStagingBuffers] =
        const_labels!("chunk_staging", NumStagingBuffers);
    const STAGING_VERTEX_LABELS: [&'static str; NumStagingBuffers] =
        const_labels!("vertex_staging", NumStagingBuffers);
    const STAGING_INDEX_LABELS: [&'static str; NumStagingBuffers] =
        const_labels!("index_staging", NumStagingBuffers);
    const STAGING_MMAT_LABELS: [&'static str; NumStagingBuffers] =
        const_labels!("mmat_staging", NumStagingBuffers);
    const COMPUTE_PASS_LABELS: [&'static str; NumStagingBuffers] =
        const_labels!("compute_pass", NumStagingBuffers);
    pub fn init(
        device: &wgpu::Device,
        chunk_buffer_size: wgpu::BufferAddress,
        vertex_buffer_size: wgpu::BufferAddress,
        index_buffer_size: wgpu::BufferAddress,
        mmat_buffer_size: wgpu::BufferAddress,
    ) -> Self {
        let layout = chunk_bind_group_layout(device);
        let pipeline = create_chunk_compute_pipeline(device, &[&layout]);

        let staging_chunk_buffers = array::from_fn(|i| {
            staging_chunk_init(device, Self::STAGING_CHUNK_LABELS[i], chunk_buffer_size)
        });
        let staging_vertex_buffers = array::from_fn(|i| {
            staging_vertex_init(device, Self::STAGING_VERTEX_LABELS[i], vertex_buffer_size)
        });
        let staging_index_buffers = array::from_fn(|i| {
            staging_index_init(device, Self::STAGING_INDEX_LABELS[i], index_buffer_size)
        });
        let staging_mmat_buffers = array::from_fn(|i| {
            staging_mmat_init(device, Self::STAGING_MMAT_LABELS[i], mmat_buffer_size)
        });
        let bind_groups = array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(("chunk_compute_bind_group_".to_owned() + &*i.to_string()).as_str()),
                layout: &layout,
                entries: &resources::utils::index_based_entries([
                    staging_chunk_buffers[i].as_entire_binding(),
                    staging_vertex_buffers[i].as_entire_binding(),
                    staging_index_buffers[i].as_entire_binding(),
                    staging_mmat_buffers[i].as_entire_binding(),
                ]),
            })
        });
        Self {
            pipeline,
            bind_groups,
            staging_chunk_buffers,
            staging_vertex_buffers,
            staging_index_buffers,
            staging_mmat_buffers,
        }
    }

    pub fn write_to_staging_chunks(
        &self,
        renderer: &Renderer<'_>,
        chunks_write: &[Vec<GPUChunkEntry>; NumStagingBuffers],
    ) {
        let mut offset = 0u64;
        for i in 0..NumStagingBuffers {
            if chunks_write[i].is_empty() {
                continue;
            }
            renderer.write_buffer(
                &self.staging_chunk_buffers[i],
                offset,
                bytemuck::cast_slice(&chunks_write[i]),
            );
            offset += chunks_write[i].len() as u64;
        }
    }

    pub fn dispatch_staging_workgroups<const NumBuffers: usize>(
        &self,
        renderer: &Renderer<'_>,
        chunk_render: &ChunkRender<NumBuffers>,
        staging_entries: &[Vec<GPUChunkEntry>; NumStagingBuffers],
        staging_targets: &[Vec<usize>; NumStagingBuffers],
    ) {
        for (staging_i, (entries, target_buffer_idx)) in staging_entries
            .iter()
            .zip(staging_targets.iter())
            .enumerate()
        {
            let mut encoder = renderer.create_encoder("chunk_mesh_compute_encoder");
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(Self::COMPUTE_PASS_LABELS[staging_i]),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, &self.bind_groups[staging_i], &[]);
                compute_pass.dispatch_workgroups(entries.len() as u32, 1, 1);
            }
            for (i, entry) in entries.iter().enumerate() {
                encoder.copy_buffer_to_buffer(
                    &self.staging_vertex_buffers[staging_i],
                    entry.header.allocation.vertex_offset as u64,
                    &chunk_render.vertex_buffers[target_buffer_idx[i]],
                    entry.header.allocation.vertex_offset as u64,
                    entry.header.allocation.vertex_size as u64 * Vertex::size() as u64,
                );
                encoder.copy_buffer_to_buffer(
                    &self.staging_index_buffers[staging_i],
                    entry.header.allocation.index_offset as u64,
                    &chunk_render.index_buffers[target_buffer_idx[i]],
                    entry.header.allocation.index_offset as u64,
                    entry.header.allocation.index_size as u64 * size_of::<Index>() as u64,
                );
                encoder.copy_buffer_to_buffer(
                    &self.staging_mmat_buffers[staging_i],
                    entry.header.slab_index as u64 * size_of::<Mat4>() as u64,
                    &chunk_render.mmat_buffer,
                    entry.header.slab_index as u64 * size_of::<Mat4>() as u64,
                    size_of::<Mat4>() as u64,
                );
            }
            renderer.queue.submit(Some(encoder.finish()));
            renderer.queue.on_submitted_work_done(move || {
                // todo
            });
        }
    }
}

pub fn chunk_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("chunk_compute_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0, // chunk block data
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1, // vertex buffer (bound as storage)
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2, // index buffer (bound as storage)
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3, // model mat buffer
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
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
        entry_point: Some("compute_main"),
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
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, // COPY_DST needed for some reason
    )
}

fn staging_vertex_init(
    device: &wgpu::Device,
    label: &str,
    size: wgpu::BufferAddress,
) -> wgpu::Buffer {
    RendererBuilder::make_buffer(
        &device,
        label,
        size,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    )
}

fn staging_index_init(
    device: &wgpu::Device,
    label: &str,
    size: wgpu::BufferAddress,
) -> wgpu::Buffer {
    RendererBuilder::make_buffer(
        &device,
        label,
        size,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    )
}

fn staging_mmat_init(
    device: &wgpu::Device,
    label: &str,
    size: wgpu::BufferAddress,
) -> wgpu::Buffer {
    RendererBuilder::make_buffer(
        &device,
        label,
        size,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    )
}
