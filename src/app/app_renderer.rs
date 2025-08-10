use crate::render::{Renderer, RendererBuilder};
use crate::{render, utils};
use std::sync::Arc;
use winit::window::Window;

pub fn make_app_renderer<'a>(window: Arc<Window>, render_distance: f32) -> Renderer<'a> {
    let renderer_builder = RendererBuilder::new(window);
    let atlas = renderer_builder.make_atlas(utils::temp::get_atlas_image());

    let max_rendered_chunks = utils::geo::max_discrete_sphere_pts(render_distance);
    let temp_size = 524288;
    
    // buffers
    let vertex_buff = renderer_builder.make_vertex_buffer(temp_size);
    let index_buff = renderer_builder.make_index_buffer(temp_size);
    let (chunk_block_buff, cbb_l) = chunk_blocks_buffer(&renderer_builder, max_rendered_chunks);
    let chunk_model_mat_buff = model_matrix_buffer(&renderer_builder, max_rendered_chunks);

    let (t_bgl, t_bg) = transform_matrices_binds(
        &renderer_builder,
        renderer_builder.view_projection_buffer.as_ref().unwrap(),
        &chunk_model_mat_buff,
    );

    // pipelines
    let cb_compute_pipeline = block_compute_pipeline(&renderer_builder, &cbb_l);
    let render_pipeline = renderer_builder.make_render_pipeline(
        render::helpers::shader::main_shader_source().into(),
        &[&atlas.texture_bind_group_layout, &t_bgl],
    );
    
    renderer_builder.build()
}

pub fn model_matrix_buffer(
    renderer_builder: &RendererBuilder,
    max_rendered_chunks: usize,
) -> wgpu::Buffer {
    // // >upper bound of max chunks to be buffered at once
    render::helpers::chunk_model::create_buffer(
        renderer_builder.device.as_ref().unwrap(),
        max_rendered_chunks,
    )
}

pub fn transform_matrices_binds(
    renderer_builder: &RendererBuilder,
    view_proj_buffer: &wgpu::Buffer,
    model_mat_buffer: &wgpu::Buffer,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let view_projection_binding = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        buffer: view_proj_buffer,
        offset: 0,
        size: std::num::NonZeroU64::new(view_proj_buffer.size()),
    });
    render::helpers::transform_matrices::create_bind_group(
        renderer_builder.device.as_ref().unwrap(),
        &render::helpers::bind_group::index_based_entries([
            view_projection_binding,              // 0
            model_mat_buffer.as_entire_binding(), // 1
        ]),
    )
}

pub fn chunk_blocks_buffer(
    renderer_builder: &RendererBuilder,
    max_rendered_chunks: usize,
) -> (wgpu::Buffer, wgpu::BindGroupLayout) {
    let block_buffer = render::helpers::chunk::create_block_buffer(
        renderer_builder.device.as_ref().unwrap(),
        max_rendered_chunks,
    );
    (
        block_buffer,
        render::helpers::chunk::block_buffer_bind_group_layout(
            renderer_builder.device.as_ref().unwrap(),
        ),
    )
}

pub fn block_compute_pipeline(
    renderer_builder: &RendererBuilder,
    block_buff_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    render::helpers::compute_pipeline::create(
        renderer_builder.device.as_ref().unwrap(),
        render::helpers::shader::meshgen_shader_source().into(),
        &[block_buff_layout],
        "block_buffer_compute_pipeline",
    )
}
