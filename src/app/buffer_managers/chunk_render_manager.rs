use super::MultiDrawInstruction;
use crate::renderer::Renderer;
use std::{array, mem};
use wgpu::wgt::DrawIndexedIndirectArgs;

pub struct ChunkRenderManager<const N: usize> {
    pub vertex_buffers: [wgpu::Buffer; N],
    pub index_buffers: [wgpu::Buffer; N],
    pub chunk_buffers: [wgpu::Buffer; N],
    pub mmat_buffers: [wgpu::Buffer; N],
}

impl<const N: usize> ChunkRenderManager<N> {
    pub fn init(
        vertex_init_fn: impl Fn(usize) -> wgpu::Buffer,
        index_init_fn: impl Fn(usize) -> wgpu::Buffer,
        chunk_init_fn: impl Fn(usize) -> wgpu::Buffer,
        mmat_init_fn: impl Fn(usize) -> wgpu::Buffer,
    ) -> Self {
        Self {
            vertex_buffers: array::from_fn(|i| vertex_init_fn(i)),
            index_buffers: array::from_fn(|i| index_init_fn(i)),
            chunk_buffers: array::from_fn(|i| chunk_init_fn(i)),
            mmat_buffers: array::from_fn(|i| mmat_init_fn(i)),
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
