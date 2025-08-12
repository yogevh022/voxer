use crate::renderer::builder::RendererAtlas;
use crate::renderer::gpu;
use crate::renderer::resources;
use crate::renderer::{Index, Renderer, RendererBuilder, Vertex};
use crate::world::types::{Chunk, GPU_CHUNK_SIZE, GPUChunkEntryHeader};
use crate::{compute, vtypes};
use glam::IVec3;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use winit::window::Window;

pub struct AppRenderer<'window> {
    pub renderer: Renderer<'window>,
    pub vertex_buff: wgpu::Buffer,
    pub index_buff: wgpu::Buffer,
    pub chunk_buff: wgpu::Buffer,
    pub chunk_mmat_buff: wgpu::Buffer,

    pub chunk_pos_to_index: HashMap<IVec3, (usize, usize, usize)>,

    pub gpu_vertex_malloc: gpu::VirtualMemAlloc,
    pub gpu_index_malloc: gpu::VirtualMemAlloc,

    pub chunk_compute_bind_group: wgpu::BindGroup,
    pub trans_mats_bind_group: wgpu::BindGroup,
    pub texture_atlas: RendererAtlas,

    pub render_pipeline: wgpu::RenderPipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
}

impl AppRenderer<'_> {
    pub fn load_chunks(&mut self, chunks: HashMap<IVec3, Chunk>) {
        let mut slice_buffer = Vec::with_capacity(chunks.len() * GPU_CHUNK_SIZE);
        chunks
            .iter()
            .enumerate()
            .for_each(|(c_idx, (c_pos, chunk))| {
                let face_count = compute::chunk::face_count(&chunk.blocks);
                let vertex_size = face_count * 4 * size_of::<Vertex>();
                let index_size = face_count * 6 * size_of::<Index>();
                let v_alloc = self.gpu_vertex_malloc.alloc(vertex_size).unwrap();
                let i_alloc = self.gpu_index_malloc.alloc(index_size).unwrap();
                self.chunk_pos_to_index
                    .insert(*c_pos, (c_idx, v_alloc, i_alloc));

                slice_buffer.extend_from_slice(bytemuck::bytes_of(&GPUChunkEntryHeader::new(
                    v_alloc,
                    i_alloc,
                    vertex_size as u32,
                    index_size as u32,
                )));
                slice_buffer.extend_from_slice(bytemuck::bytes_of(&chunk.blocks));
            });

        self.renderer
            .write_buffer(&self.chunk_buff, 0, bytemuck::cast_slice(&slice_buffer));

        // println!("{:?}", self.gpu_vertex_malloc);
    }

    pub fn unload_chunks(&mut self, chunks: HashSet<IVec3>) {
        for c_pos in chunks {
            let (c_idx, v_idx, i_idx) = self.chunk_pos_to_index.remove(&c_pos).unwrap();
            self.gpu_vertex_malloc.free(v_idx);
            self.gpu_index_malloc.free(i_idx);
            self.renderer.write_buffer(
                &self.chunk_buff,
                (c_idx * GPU_CHUNK_SIZE) as u64,
                bytemuck::cast_slice(&[0u32]), // make the 'exists' bit 0
            )
        }
    }

    pub fn encode_compute_chunks(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("compute_pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.chunk_compute_bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }

    pub fn encode_render_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera: &vtypes::Camera,
    ) {
        let mut render_pass =
            resources::render_pass::begin(encoder, view, &self.renderer.depth_texture_view);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture_atlas.bind_group, &[]);

        let view_proj = camera.get_view_projection().to_cols_array();
        self.renderer.write_buffer(
            &self.renderer.view_projection_buffer,
            0u64,
            bytemuck::cast_slice(&[view_proj]),
        );
        render_pass.set_vertex_buffer(0, self.vertex_buff.slice(..));
        render_pass.set_index_buffer(self.index_buff.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(1, &self.trans_mats_bind_group, &[]);
        render_pass.draw_indexed(0..1000, 0, 0..1);
    }

    pub fn render(&mut self, camera: &vtypes::Camera) -> Result<(), wgpu::SurfaceError> {
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

        // self.encode_compute_chunks(&mut encoder);
        self.encode_render_pass(&mut encoder, &view, camera);

        self.renderer.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

pub fn make_app_renderer<'a>(window: Arc<Window>, render_distance: f32) -> AppRenderer<'a> {
    let renderer_builder = RendererBuilder::new(window);
    let atlas = renderer_builder.make_atlas(get_atlas_image());

    // >upper bound of max chunks to be buffered at once
    let max_rendered_chunks = compute::geo::max_discrete_sphere_pts(render_distance);
    let temp_size = (compute::MIB * 128) as u64;

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
        resources::shader::main_shader_source().into(),
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
                entries: &resources::bind_group::index_based_entries([
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
        chunk_pos_to_index: HashMap::new(),
        gpu_vertex_malloc: gpu::VirtualMemAlloc::new(temp_size as usize),
        gpu_index_malloc: gpu::VirtualMemAlloc::new(temp_size as usize),
        chunk_compute_bind_group,
        trans_mats_bind_group: t_bg,
        texture_atlas: atlas,
        render_pipeline,
        compute_pipeline: cb_compute_pipeline,
    }
}

pub fn model_matrix_buffer(
    renderer_builder: &RendererBuilder,
    max_rendered_chunks: usize,
) -> wgpu::Buffer {
    resources::chunk_model::create_buffer(
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
    resources::transform_matrices::create_bind_group(
        renderer_builder.device.as_ref().unwrap(),
        &resources::bind_group::index_based_entries([
            view_projection_binding,              // 0
            model_mat_buffer.as_entire_binding(), // 1
        ]),
    )
}

pub fn chunk_blocks_buffer(
    renderer_builder: &RendererBuilder,
    max_rendered_chunks: usize,
) -> (wgpu::Buffer, wgpu::BindGroupLayout) {
    let block_buffer = resources::chunk::create_block_buffer(
        renderer_builder.device.as_ref().unwrap(),
        max_rendered_chunks,
    );
    (
        block_buffer,
        resources::chunk::block_buffer_bind_group_layout(renderer_builder.device.as_ref().unwrap()),
    )
}

pub fn block_compute_pipeline(
    renderer_builder: &RendererBuilder,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    let shader = resources::shader::create(
        renderer_builder.device.as_ref().unwrap(),
        resources::shader::meshgen_shader_source().into(),
    );
    resources::pipeline::create_compute(
        renderer_builder.device.as_ref().unwrap(),
        bind_group_layouts,
        &shader,
        "block_buffer_compute_pipeline",
    )
}

pub fn get_atlas_image() -> image::RgbaImage {
    let atlas_image =
        image::open("src/renderer/texture/images/atlas.png").expect("failed to load atlas.png");
    atlas_image.to_rgba8()
}
