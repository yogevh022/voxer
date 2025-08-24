use super::MultiDrawInstruction;
use crate::compute;
use crate::renderer::{Renderer, resources};
use std::{array, mem};
use wgpu::wgt::DrawIndexedIndirectArgs;

pub struct ChunkRenderManager<const N: usize> {
    pub vertex_buffers: [wgpu::Buffer; N],
    pub index_buffers: [wgpu::Buffer; N],
    pub mmat_buffers: [wgpu::Buffer; N],
    pub mmat_bind_groups: [wgpu::BindGroup; N], // fixme shouldnt be pub?
}

impl<const N: usize> ChunkRenderManager<N> {
    pub fn init(
        renderer: &Renderer<'_>,
        vertex_init_fn: impl Fn(usize) -> wgpu::Buffer,
        index_init_fn: impl Fn(usize) -> wgpu::Buffer,
        mmat_init_fn: impl Fn(usize) -> wgpu::Buffer,
    ) -> Self {
        let vertex_buffers = array::from_fn(|i| vertex_init_fn(i));
        let index_buffers = array::from_fn(|i| index_init_fn(i));
        let mmat_buffers = array::from_fn(|i| mmat_init_fn(i));

        let mmat_bind_groups = array::from_fn(|i| {
            renderer
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("transform_matrices_bind_group"),
                    layout: &renderer.layouts.mmat,
                    entries: &resources::utils::index_based_entries([
                        mmat_buffers[i].as_entire_binding(), // 0
                    ]),
                })
        });
        Self {
            vertex_buffers,
            index_buffers,
            mmat_buffers,
            mmat_bind_groups,
        }
    }

    pub fn write_commands_to_indirect_buffer(
        &self,
        renderer: &Renderer<'_>,
        buffer_commands: &[Vec<DrawIndexedIndirectArgs>; N],
    ) -> [MultiDrawInstruction; N] {
        let mut command_count = 0;
        let mut indirect_offsets = [MultiDrawInstruction::default(); N];
        for (i, command) in buffer_commands.iter().enumerate() {
            indirect_offsets[i] = MultiDrawInstruction {
                offset: command_count * size_of::<DrawIndexedIndirectArgs>(),
                count: command.len(),
            };
            command_count += command.len();
        }

        let flat_commands = buffer_commands
            .iter()
            .flat_map(|x| x.iter().copied())
            .collect::<Vec<_>>();

        renderer.write_buffer(
            &renderer.indirect_buffer,
            0,
            bytemuck::cast_slice(&flat_commands),
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
            render_pass.set_bind_group(2, &self.mmat_bind_groups[i], &[]);

            render_pass.multi_draw_indexed_indirect(
                &renderer.indirect_buffer,
                multi_draw_instructions[i].offset as u64,
                multi_draw_instructions[i].count as u32,
            );
        }
    }
}
