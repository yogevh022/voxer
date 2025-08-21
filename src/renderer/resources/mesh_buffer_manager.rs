use crate::renderer::Renderer;
use std::{array, mem};
use wgpu::wgt::DrawIndexedIndirectArgs;

#[derive(Clone, Copy)]
pub enum BufferType {
    Vertex,
    Index,
}

#[derive(Clone, Copy)]
pub struct ComputeInstruction {
    pub target_staging_buffer: usize,
    pub buffer_type: BufferType,
    pub byte_offset: usize,
    pub byte_length: usize,
}

#[derive(Default, Clone, Copy)]
pub struct MultiDrawInstruction {
    pub offset: usize,
    pub count: usize,
}

pub struct MeshBufferManager<const N: usize, const S: usize> {
    vertex_buffers: [wgpu::Buffer; N],
    index_buffers: [wgpu::Buffer; N],

    staging_data_buffers: [wgpu::Buffer; S],
    staging_vertex_buffers: [wgpu::Buffer; S],
    staging_index_buffers: [wgpu::Buffer; S],
}

impl<const N: usize, const S: usize> MeshBufferManager<N, S> {
    pub fn init(
        vertex_init_fn: impl Fn(usize) -> wgpu::Buffer,
        index_init_fn: impl Fn(usize) -> wgpu::Buffer,
        staging_init_fn: impl Fn(usize) -> wgpu::Buffer,
    ) -> Self {
        Self {
            vertex_buffers: array::from_fn(|i| vertex_init_fn(i)),
            index_buffers: array::from_fn(|i| index_init_fn(i)),
            staging_data_buffers: array::from_fn(|i| staging_init_fn(i)),
            staging_vertex_buffers: array::from_fn(|i| staging_init_fn(i + S)),
            staging_index_buffers: array::from_fn(|i| staging_init_fn(i + S + S)),
        }
    }

    fn create_encoder(renderer: &Renderer<'_>, label: &str) -> wgpu::CommandEncoder {
        renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }

    pub fn dispatch_staging_workgroups(
        &self,
        renderer: &Renderer<'_>,
        compute_pipeline: &wgpu::ComputePipeline,
        compute_bind_groups: [wgpu::BindGroup; N],
        compute_commands: [Vec<ComputeInstruction>; N],
    ) {
        for (i, compute_instruction) in compute_commands.into_iter().enumerate() {
            let mut encoder = Self::create_encoder(
                renderer,
                ("dispatch_staging_".to_owned() + &*i.to_string()).as_str(),
            );
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(("compute_pass_".to_owned() + &*i.to_string()).as_str()),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&compute_pipeline);
                compute_pass.set_bind_group(0, &compute_bind_groups[i], &[]);
                compute_pass.dispatch_workgroups(compute_instruction.len() as u32, i as u32, 1);
            }
            for inst in compute_instruction {
                match inst.buffer_type {
                    BufferType::Vertex => {
                        encoder.copy_buffer_to_buffer(
                            &self.staging_vertex_buffers[inst.target_staging_buffer],
                            inst.byte_offset as u64,
                            &self.vertex_buffers[i],
                            inst.byte_offset as u64,
                            inst.byte_length as u64,
                        );
                    }
                    BufferType::Index => {
                        encoder.copy_buffer_to_buffer(
                            &self.staging_index_buffers[inst.target_staging_buffer],
                            inst.byte_offset as u64,
                            &self.index_buffers[i],
                            inst.byte_offset as u64,
                            inst.byte_length as u64,
                        );
                    }
                }
            }
            renderer.queue.submit(Some(encoder.finish()));
            renderer.queue.on_submitted_work_done(move || {
                // todo on submit
            });
        }
    }

    pub fn write_commands_to_indirect_buffer(
        &self,
        renderer: &Renderer<'_>,
        buffer_commands: [Vec<DrawIndexedIndirectArgs>; N],
    ) -> [MultiDrawInstruction; N] {
        let mut indirect_commands: Vec<_> = Vec::new();
        let mut indirect_offsets = [MultiDrawInstruction::default(); N];
        for (i, command) in buffer_commands.into_iter().enumerate() {
            indirect_offsets[i] = MultiDrawInstruction {
                offset: indirect_commands.len(),
                count: command.len(),
            };
            indirect_commands.extend(command.into_iter());
        }
        renderer.write_buffer(
            &renderer.indirect_buffer,
            0,
            bytemuck::cast_slice(&indirect_commands),
        );
        indirect_offsets
    }

    pub fn multi_draw(
        &self,
        renderer: &Renderer<'_>,
        render_pass: &mut wgpu::RenderPass,
        multi_draw_instructions: [MultiDrawInstruction; N],
    ) {
        for i in 0..N {
            render_pass.set_vertex_buffer(0, self.vertex_buffers[i].slice(..));
            render_pass
                .set_index_buffer(self.index_buffers[i].slice(..), wgpu::IndexFormat::Uint32);

            render_pass.multi_draw_indexed_indirect(
                &renderer.indirect_buffer,
                multi_draw_instructions[i].offset as u64,
                multi_draw_instructions[i].count as u32,
            );
        }
    }
}
