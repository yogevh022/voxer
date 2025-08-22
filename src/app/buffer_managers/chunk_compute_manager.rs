use super::{BufferType, ComputeInstruction, WriteInstruction};
use crate::renderer::{Renderer, resources};
use std::array;

pub struct ChunkComputeManager<const S: usize> {
    pipeline: wgpu::ComputePipeline,
    bind_groups: [wgpu::BindGroup; S],
    staging_chunk_buffers: [wgpu::Buffer; S],
    staging_vertex_buffers: [wgpu::Buffer; S],
    staging_index_buffers: [wgpu::Buffer; S],
    staging_mmat_buffers: [wgpu::Buffer; S],
}

impl<const S: usize> ChunkComputeManager<S> {
    pub fn init(
        device: &wgpu::Device,
        staging_chunk_init_fn: impl Fn(usize) -> wgpu::Buffer,
        staging_vertex_init_fn: impl Fn(usize) -> wgpu::Buffer,
        staging_index_init_fn: impl Fn(usize) -> wgpu::Buffer,
        staging_mmat_init_fn: impl Fn(usize) -> wgpu::Buffer,
    ) -> Self {
        let layout = chunk_bind_group_layout(device);
        let pipeline = create_chunk_compute_pipeline(device, &[&layout]);
        let staging_chunk_buffers = array::from_fn(|i| staging_chunk_init_fn(i));
        let staging_vertex_buffers = array::from_fn(|i| staging_vertex_init_fn(i));
        let staging_index_buffers = array::from_fn(|i| staging_index_init_fn(i));
        let staging_mmat_buffers = array::from_fn(|i| staging_mmat_init_fn(i));
        let bind_groups = array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(("chunk_compute_bind_group_".to_owned() + &*i.to_string()).as_str()),
                layout: &layout,
                entries: &resources::bind_group::index_based_entries([
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

    pub fn write_to_staging_chunks(&self, renderer: &Renderer<'_>, write_instructions: &[WriteInstruction<'_>; S]) {
        for i in 0..S {
            renderer.write_buffer(
                &self.staging_chunk_buffers[i],
                write_instructions[i].offset,
                write_instructions[i].bytes,
            );
        }
    }

    pub fn dispatch_staging_workgroups<const N: usize>(
        &self,
        renderer: &Renderer<'_>,
        chunk_buffers: &[wgpu::Buffer; N],
        mmat_buffers: &[wgpu::Buffer; N],
        vertex_buffers: &[wgpu::Buffer; N],
        index_buffers: &[wgpu::Buffer; N],
        compute_commands: [Vec<ComputeInstruction>; N],
    ) {
        for (i, compute_instruction) in compute_commands.into_iter().enumerate() {
            let mut encoder = create_encoder(
                renderer,
                ("dispatch_staging_".to_owned() + &*i.to_string()).as_str(),
            );
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(("compute_pass_".to_owned() + &*i.to_string()).as_str()),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, &self.bind_groups[i], &[]);
                compute_pass.dispatch_workgroups(compute_instruction.len() as u32, i as u32, 1);
            }
            for inst in compute_instruction {
                match inst.buffer_type {
                    BufferType::Vertex => {
                        encoder.copy_buffer_to_buffer(
                            &self.staging_vertex_buffers[inst.target_staging_buffer],
                            inst.byte_offset as u64,
                            &vertex_buffers[i],
                            inst.byte_offset as u64,
                            inst.byte_length as u64,
                        );
                    }
                    BufferType::Index => {
                        encoder.copy_buffer_to_buffer(
                            &self.staging_index_buffers[inst.target_staging_buffer],
                            inst.byte_offset as u64,
                            &index_buffers[i],
                            inst.byte_offset as u64,
                            inst.byte_length as u64,
                        );
                    }
                    BufferType::MMat => encoder.copy_buffer_to_buffer(
                        &self.staging_mmat_buffers[inst.target_staging_buffer],
                        inst.byte_offset as u64,
                        &mmat_buffers[i],
                        inst.byte_offset as u64,
                        inst.byte_length as u64,
                    ),
                    BufferType::Chunk => encoder.copy_buffer_to_buffer(
                        &self.staging_chunk_buffers[inst.target_staging_buffer],
                        inst.byte_offset as u64,
                        &chunk_buffers[i],
                        inst.byte_offset as u64,
                        inst.byte_length as u64,
                    ),
                }
            }
            renderer.queue.submit(Some(encoder.finish()));
            renderer.queue.on_submitted_work_done(move || {
                // todo on submit
            });
        }
    }
}

fn create_encoder(renderer: &Renderer<'_>, label: &str) -> wgpu::CommandEncoder {
    // fixme perhaps move this
    renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
}

pub fn chunk_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("chunk_compute_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0, // chunk block data
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
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
    resources::pipeline::create_compute(
        device,
        bind_group_layouts,
        &shader,
        "chunk_compute_pipeline",
    )
}
