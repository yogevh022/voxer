use crate::app::buffer_managers::MultiDrawInstruction;
use crate::renderer::{Renderer, RendererBuilder, resources};
use std::collections::HashMap;
use std::{array};
use wgpu::wgt::DrawIndexedIndirectArgs;

pub struct ChunkRender<const N: usize> {
    pub vertex_buffers: [wgpu::Buffer; N],
    pub index_buffers: [wgpu::Buffer; N],
    pub mmat_buffer: wgpu::Buffer,
    mmat_bind_group: wgpu::BindGroup,
}

impl<const N: usize> ChunkRender<N> {
    pub fn init(
        renderer: &Renderer<'_>,
        vertex_buffer_size: wgpu::BufferAddress,
        index_buffer_size: wgpu::BufferAddress,
        mmat_buffer_size: wgpu::BufferAddress,
    ) -> Self {
        let vertex_buffers =
            array::from_fn(|i| vertex_init(&renderer.device, &format!("vertex_{}", i), vertex_buffer_size));
        let index_buffers =
            array::from_fn(|i| index_init(&renderer.device, &format!("index_{}", i), index_buffer_size));
        let mmat_buffer = mmat_init(&renderer.device, "mmat_0", mmat_buffer_size);
        let mmat_bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("transform_matrices_bind_group"),
                layout: &renderer.layouts.mmat,
                entries: &resources::utils::index_based_entries([
                    mmat_buffer.as_entire_binding(), // 0
                ]),
            });
        Self {
            vertex_buffers,
            index_buffers,
            mmat_buffer,
            mmat_bind_group,
        }
    }

    pub fn write_commands_to_indirect_buffer(
        &self,
        renderer: &Renderer<'_>,
        buffer_commands: &[HashMap<u32, DrawIndexedIndirectArgs>; N],
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
            .flat_map(|x| x.values().copied())
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
        render_pass.set_bind_group(2, &self.mmat_bind_group, &[]);
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

fn vertex_init(device: &wgpu::Device, label: &str, size: wgpu::BufferAddress) -> wgpu::Buffer {
    RendererBuilder::make_buffer(
        &device,
        label,
        size,
        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    )
}

fn index_init(device: &wgpu::Device, label: &str, size: wgpu::BufferAddress) -> wgpu::Buffer {
    RendererBuilder::make_buffer(
        &device,
        label,
        size,
        wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
    )
}

fn mmat_init(device: &wgpu::Device, label: &str, size: wgpu::BufferAddress) -> wgpu::Buffer {
    RendererBuilder::make_buffer(
        &device,
        label,
        size,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
    )
}
