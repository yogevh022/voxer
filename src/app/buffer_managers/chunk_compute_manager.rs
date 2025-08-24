use super::{ComputeInstruction, WriteInstruction};
use crate::app::app_renderer::DrawDelta;
use crate::renderer::{Renderer, resources};
use glam::IVec3;
use parking_lot::RwLock;
use std::array;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DrawIndexedIndirectArgs;

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

    pub fn write_to_staging_chunks<const N: usize>(
        &self,
        renderer: &Renderer<'_>,
        write_instructions: [WriteInstruction<'_>; N],
    ) {
        for i in 0..N {
            if write_instructions[i].bytes.is_empty() {
                continue;
            }
            renderer.write_buffer(
                &self.staging_chunk_buffers[write_instructions[i].staging_index],
                write_instructions[i].offset,
                write_instructions[i].bytes,
            );
        }
    }

    pub fn dispatch_staging_workgroups<const N: usize>(
        &self,
        renderer: &Renderer<'_>,
        mmat_buffers: &[wgpu::Buffer; N],
        vertex_buffers: &[wgpu::Buffer; N],
        index_buffers: &[wgpu::Buffer; N],
        compute_instructions: [Vec<ComputeInstruction>; S],
        mut local_draw_delta: [[HashMap<u32, DrawIndexedIndirectArgs>; N]; S],
        draw_delta: &Arc<RwLock<DrawDelta<N>>>,
    ) {
        for (staging_i, compute_instruction) in compute_instructions.into_iter().enumerate() {
            let mut encoder =
                renderer
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("render_encoder"),
                    });
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(("compute_pass_".to_owned() + &*staging_i.to_string()).as_str()),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, &self.bind_groups[staging_i], &[]);
                compute_pass.dispatch_workgroups(compute_instruction.len() as u32, 1, 1);
            }
            for inst in compute_instruction {
                encoder.copy_buffer_to_buffer(
                    &self.staging_vertex_buffers[staging_i],
                    inst.vertex_offset_bytes,
                    &vertex_buffers[inst.target_buffer],
                    inst.vertex_offset_bytes,
                    inst.vertex_size_bytes,
                );
                encoder.copy_buffer_to_buffer(
                    &self.staging_index_buffers[staging_i],
                    inst.index_offset_bytes,
                    &index_buffers[inst.target_buffer],
                    inst.index_offset_bytes,
                    inst.index_size_bytes,
                );
                encoder.copy_buffer_to_buffer(
                    &self.staging_mmat_buffers[staging_i],
                    inst.mmat_offset_bytes,
                    &mmat_buffers[inst.target_buffer],
                    inst.mmat_offset_bytes,
                    inst.mmat_size_bytes,
                );
            }
            renderer.queue.submit(Some(encoder.finish()));
            let mut staging_i_delta: [HashMap<_, _>; N] = array::from_fn(|_| HashMap::new());
            std::mem::swap(&mut staging_i_delta, &mut local_draw_delta[staging_i]);
            let target_draw_args_ref = draw_delta.clone();
            renderer.queue.on_submitted_work_done(move || {
                let mut draw_args_lock = target_draw_args_ref.write();
                for (i, args) in staging_i_delta.into_iter().enumerate() {
                    draw_args_lock.args[i].extend(args);
                }
                draw_args_lock.changed = true;
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
