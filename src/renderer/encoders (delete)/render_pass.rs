use crate::render::helpers;
use crate::render::renderer::resources::RenderResources;
use glam::Mat4;

pub struct RenderPassEncodeContext<'a> {
    pub view: wgpu::TextureView,
    pub pipeline: &'a wgpu::RenderPipeline,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub resources: &'a RenderResources,
    pub view_projection: Mat4,
}

pub fn encode(encoder: &mut wgpu::CommandEncoder, context: RenderPassEncodeContext) {
    let mut render_pass = helpers::render_pass::begin(
        encoder,
        &context.view,
        &context.resources.depth_texture_view,
    );
    render_pass.set_pipeline(&context.pipeline);
    render_pass.set_bind_group(0, &context.resources.terrain.atlas_bind_group, &[]);

    let view_proj = context.view_projection.to_cols_array();
    context.queue.write_buffer(
        &context.resources.transform.uniform_buffer,
        0u64,
        bytemuck::cast_slice(&[view_proj]),
    );

    // for (i, (c_pos, chunk_buffer_entry)) in context.resources.chunk_pool.iter().enumerate() {
    //     let idx = chunk_buffer_entry.index_offset;
    //     render_pass.set_bind_group(1, &context.resources.transform.bind_group, &[]);
    //     render_pass.set_vertex_buffer(
    //         0,
    //         context
    //             .resources
    //             .chunk_pool
    //             .get_index(i)
    //             .1
    //             .mesh_buffers
    //             .vertex
    //             .slice(..),
    //     );
    //     render_pass.set_index_buffer(
    //         context
    //             .resources
    //             .chunk_pool
    //             .get_index(i)
    //             .1
    //             .mesh_buffers
    //             .index
    //             .slice(..),
    //         wgpu::IndexFormat::Uint32,
    //     );
    //     render_pass.draw_indexed(0..idx, 0, (i as u32)..(i as u32 + 1));
    // }
}
