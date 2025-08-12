use crate::render::alloc::MemoryAllocator;
use crate::render::helpers::bind_group::index_based_entries;
use crate::render::types::{Index, Vertex};
use crate::render::{Renderer, RendererBuilder};
use crate::world::types::{Block, CHUNK_VOLUME, ChunkBlocks, GPUChunkEntry};
use crate::{compute, render, types, utils};
use std::sync::Arc;
use winit::window::Window;

pub struct AppRenderer<'window> {
    pub renderer: Renderer<'window>,
    pub vertex_buff: wgpu::Buffer,
    pub index_buff: wgpu::Buffer,
    pub chunk_buff: wgpu::Buffer,
    pub chunk_mmat_buff: wgpu::Buffer,

    pub gpu_vertex_malloc: MemoryAllocator,
    pub gpu_index_malloc: MemoryAllocator,

    pub chunk_compute_bind_group: wgpu::BindGroup,
    pub trans_mats_bind_group: wgpu::BindGroup,

    pub render_pipeline: wgpu::RenderPipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
}

impl AppRenderer<'_> {
    pub fn load_chunks(&mut self, insert_offset: usize, chunks: &[&ChunkBlocks]) {
        let gpu_chunk_entries: Vec<GPUChunkEntry> = chunks
            .into_iter()
            .map(|blocks| {
                let face_count = compute::chunk::face_count(blocks);
                let vertex_count = face_count * 4 * size_of::<Vertex>();
                let index_count = face_count * 6 * size_of::<Index>();
                let vertex_offset = self.gpu_vertex_malloc.alloc_ff(vertex_count).unwrap();
                let index_offset = self.gpu_index_malloc.alloc_ff(index_count).unwrap();
                GPUChunkEntry::new(blocks, vertex_offset, index_offset, face_count)
            })
            .collect();
        let raw_chunks = unsafe {
            std::slice::from_raw_parts(
                gpu_chunk_entries.as_ptr() as *const u8,
                gpu_chunk_entries.len() * size_of::<GPUChunkEntry>(),
            )
        };
        self.renderer.write_buffer(
            &self.chunk_buff,
            (insert_offset * size_of::<GPUChunkEntry>()) as u64,
            bytemuck::cast_slice(raw_chunks),
        )
    }

    pub fn unload_chunks(&mut self, chunks: Vec<usize>) {
        todo!();
        for i in chunks {
            // self.gpu_vertex_malloc.free()
            self.renderer.write_buffer(
                &self.chunk_buff,
                (i * size_of::<GPUChunkEntry>()) as u64,
                bytemuck::cast_slice(&[0u8]), // make the 'exists' bit 0
            )
        }
    }

    pub fn compute_chunks(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("compute_pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.chunk_compute_bind_group, &[]);
    }

    pub fn render(&mut self, camera: &types::Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });

        self.renderer.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

pub fn make_app_renderer<'a>(window: Arc<Window>, render_distance: f32) -> AppRenderer<'a> {
    let renderer_builder = RendererBuilder::new(window);
    let atlas = renderer_builder.make_atlas(utils::temp::get_atlas_image());

    let max_rendered_chunks = utils::geo::max_discrete_sphere_pts(render_distance);
    let temp_size = 524288;

    // buffers
    let vertex_buff = renderer_builder.make_vertex_buffer(temp_size);
    let index_buff = renderer_builder.make_index_buffer(temp_size);
    let (chunk_data_buff, cbb_l) = chunk_blocks_buffer(&renderer_builder, max_rendered_chunks);
    let chunk_model_mat_buff = model_matrix_buffer(&renderer_builder, max_rendered_chunks);

    let (t_bgl, t_bg) = transform_matrices_binds(
        &renderer_builder,
        renderer_builder.view_projection_buffer.as_ref().unwrap(),
        &chunk_model_mat_buff,
    );

    // pipelines
    let cb_compute_pipeline = block_compute_pipeline(&renderer_builder, &[&cbb_l]);
    let render_pipeline = renderer_builder.make_render_pipeline(
        render::helpers::shader::main_shader_source().into(),
        &[&atlas.texture_bind_group_layout, &t_bgl],
    );

    let chunk_compute_bind_group =
        renderer_builder
            .device
            .as_ref()
            .unwrap()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("chunk_compute_bind_group"),
                layout: &cbb_l,
                entries: &index_based_entries([
                    chunk_data_buff.as_entire_binding(),
                    vertex_buff.as_entire_binding(),
                    index_buff.as_entire_binding(),
                    chunk_model_mat_buff.as_entire_binding(),
                ]),
            });

    let renderer = renderer_builder.build();

    AppRenderer {
        renderer,
        vertex_buff,
        index_buff,
        chunk_buff: chunk_data_buff,
        chunk_mmat_buff: chunk_model_mat_buff,
        gpu_vertex_malloc: MemoryAllocator::new(temp_size as usize),
        gpu_index_malloc: MemoryAllocator::new(temp_size as usize),
        chunk_compute_bind_group,
        trans_mats_bind_group: t_bg,
        render_pipeline,
        compute_pipeline: cb_compute_pipeline,
    }
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
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    render::helpers::compute_pipeline::create(
        renderer_builder.device.as_ref().unwrap(),
        render::helpers::shader::meshgen_shader_source().into(),
        bind_group_layouts,
        "block_buffer_compute_pipeline",
    )
}
