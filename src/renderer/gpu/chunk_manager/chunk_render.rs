use crate::renderer::resources::vg_buffer_resource::VgBufferResource;
use crate::renderer::gpu::chunk_manager::BufferDrawArgs;
use crate::renderer::{Renderer, resources, VxDrawIndirectBatch};
use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Device, ShaderStages};

pub struct ChunkRender {
    pub face_data_buffer: VgBufferResource,
    pub bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl ChunkRender {
    pub fn init(
        device: &Device,
        view_projection_buffer: &VgBufferResource,
        face_data_buffer_size: wgpu::BufferAddress,
    ) -> Self {
        let face_data_buffer = VgBufferResource::new(
            &device,
            "Chunk Face Data Buffer",
            face_data_buffer_size,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        );

        let (layout, bind_group) =
            chunk_render_bind_group(&device, view_projection_buffer, &face_data_buffer);

        Self {
            face_data_buffer,
            bind_group,
            bind_group_layout: layout,
        }
    }

    pub fn write_indirect_draw_args(
        &self,
        renderer: &Renderer<'_>,
        buffer_draw_args: &BufferDrawArgs,
    ) {
        // todo encode batch on first iter?
        let draw_indirect_batch = VxDrawIndirectBatch::from_iter(buffer_draw_args.values());
        renderer.write_buffer(
            &renderer.indirect_buffer,
            0,
            bytemuck::cast_slice(&draw_indirect_batch.encode(renderer.adapter_info().backend)),
        );
    }

    pub fn draw(&self, renderer: &Renderer<'_>, render_pass: &mut wgpu::RenderPass, count: u32) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        if count != 0 {
            render_pass.multi_draw_indirect(&renderer.indirect_buffer, 0, count);
        }
    }
}

fn chunk_render_bind_group(
    device: &Device,
    view_projection_buffer: &VgBufferResource,
    face_data_buffer: &VgBufferResource,
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
