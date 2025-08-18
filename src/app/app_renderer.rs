use crate::renderer::builder::RendererAtlas;
use crate::renderer::gpu::{ChunkVMA, GPUChunkEntryBuffer, GPUChunkEntryHeader, VirtualMemAlloc};
use crate::renderer::resources;
use crate::renderer::{Index, Renderer, RendererBuilder, Vertex};
use crate::world::types::Chunk;
use crate::{compute, vtypes};
use glam::IVec3;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use winit::window::Window;

const VOID_MESH_OFFSET: usize = 8;

pub struct AppRenderer<'window> {
    pub renderer: Renderer<'window>,
    pub global_encoder: wgpu::CommandEncoder,

    pub indirect_buff: wgpu::Buffer,
    pub vertex_buff: wgpu::Buffer,
    pub index_buff: wgpu::Buffer,
    pub chunk_buff: wgpu::Buffer,

    pub loaded_chunks: HashMap<IVec3, GPUChunkEntryHeader>,

    pub chunk_malloc: ChunkVMA,

    pub chunk_compute_bind_group: wgpu::BindGroup,
    pub trans_mats_bind_group: wgpu::BindGroup,
    pub texture_atlas: RendererAtlas,

    pub render_pipeline: wgpu::RenderPipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
}

impl AppRenderer<'_> {
    pub fn write_new_chunks(&mut self, chunks: Vec<(usize, IVec3, Chunk)>) {
        let chunks_count = chunks.len();
        let mut chunk_entries = GPUChunkEntryBuffer::new(chunks_count);
        for (slab_index, chunk_pos, chunk) in chunks.into_iter() {
            let header = GPUChunkEntryHeader::from_chunk_data(
                &mut self.chunk_malloc,
                &chunk,
                chunk_pos,
                slab_index as u32,
            );
            self.loaded_chunks.insert(chunk_pos, header.clone());

            chunk_entries.insert(header, chunk.blocks);
        }

        self.renderer.write_buffer(
            &self.chunk_buff,
            0,
            &bytemuck::bytes_of(&(chunks_count as u32)),
        );
        self.renderer
            .write_buffer(&self.chunk_buff, 16, &bytemuck::cast_slice(&chunk_entries));

        // self.gpu_vertex_malloc.draw_cli();
    }

    pub fn unload_chunks(&mut self, chunks: Vec<IVec3>) {
        for c_pos in chunks {
            let chunk_entry = self.loaded_chunks.remove(&c_pos).unwrap();
            self.chunk_malloc
                .vertex
                .free(chunk_entry.vertex_offset as usize);
            self.chunk_malloc
                .index
                .free(chunk_entry.index_offset as usize);
        }
        // self.gpu_vertex_malloc.draw_cli();
    }

    pub fn compute_chunks(&mut self) {
        let mut compute_pass =
            self.global_encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("compute_pass"),
                    timestamp_writes: None,
                });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.chunk_compute_bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }

    pub fn encode_render_pass(&mut self, view: &wgpu::TextureView, camera: &vtypes::Camera) {
        let mut render_pass = resources::render_pass::begin(
            &mut self.global_encoder,
            view,
            &self.renderer.depth_texture_view,
        );
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

        if !self.loaded_chunks.is_empty() {
            let buffer_commands = self
                .loaded_chunks
                .values()
                .map(|header| header.draw_indexed_indirect_args())
                .collect::<Vec<_>>();
            self.renderer.write_buffer(
                &self.indirect_buff,
                0,
                &bytemuck::cast_slice(&buffer_commands),
            );
            // dbg!(&self.loaded_chunks.iter().collect::<Vec<_>>());
            render_pass.multi_draw_indexed_indirect(
                &self.indirect_buff,
                0,
                buffer_commands.len() as u32,
            );
        }
    }

    fn take_global_encoder(&mut self) -> wgpu::CommandEncoder {
        // returns owned global encoder, and replaces it with a new one
        let mut encoder = create_encoder(&self.renderer.device);
        std::mem::swap(&mut self.global_encoder, &mut encoder);
        encoder
    }

    pub fn render(&mut self, camera: &vtypes::Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.encode_render_pass(&view, camera);

        let encoder = self.take_global_encoder();

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
    let (chunk_buff, chunk_buff_layout) =
        chunk_blocks_buffer(&renderer_builder, temp_size as usize);
    let chunk_mmat_buff = model_matrix_buffer(&renderer_builder, max_rendered_chunks);

    let (transform_mats_bgl, transform_mats_bg) = transform_matrices_binds(
        &renderer_builder,
        renderer_builder.view_projection_buffer.as_ref().unwrap(),
        &chunk_mmat_buff,
    );

    // pipelines
    let cb_compute_pipeline = block_compute_pipeline(&renderer_builder, &[&chunk_buff_layout]);
    let render_pipeline = renderer_builder.make_render_pipeline(
        resources::shader::main_shader().into(),
        &[&atlas.texture_bind_group_layout, &transform_mats_bgl],
    );

    let chunk_compute_bind_group =
        renderer_builder
            .device
            .as_ref()
            .unwrap()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("chunk_compute_bind_group"),
                layout: &chunk_buff_layout,
                entries: &resources::bind_group::index_based_entries([
                    chunk_buff.as_entire_binding(),
                    vertex_buff.as_entire_binding(),
                    index_buff.as_entire_binding(),
                    chunk_mmat_buff.as_entire_binding(),
                ]),
            });

    let renderer = renderer_builder.build();
    let encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

    let indirect_buff = indirect_buffer(&renderer.device, 250 * compute::KIB as u64);

    let chunk_malloc = ChunkVMA {
        vertex: VirtualMemAlloc::new(temp_size as usize / Vertex::size(), VOID_MESH_OFFSET),
        index: VirtualMemAlloc::new(temp_size as usize / size_of::<Index>(), VOID_MESH_OFFSET),
    };

    AppRenderer {
        renderer,
        global_encoder: encoder,
        indirect_buff,
        vertex_buff,
        index_buff,
        chunk_buff,
        loaded_chunks: HashMap::new(),
        chunk_malloc,
        chunk_compute_bind_group,
        trans_mats_bind_group: transform_mats_bg,
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
    size: usize,
) -> (wgpu::Buffer, wgpu::BindGroupLayout) {
    let block_buffer =
        resources::chunk::create_chunk_buffer(renderer_builder.device.as_ref().unwrap(), size);
    (
        block_buffer,
        resources::chunk::chunk_buffer_bind_group_layout(renderer_builder.device.as_ref().unwrap()),
    )
}

pub fn block_compute_pipeline(
    renderer_builder: &RendererBuilder,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    let shader = resources::shader::create(
        renderer_builder.device.as_ref().unwrap(),
        resources::shader::chunk_meshing().into(),
    );
    resources::pipeline::create_compute(
        renderer_builder.device.as_ref().unwrap(),
        bind_group_layouts,
        &shader,
        "block_buffer_compute_pipeline",
    )
}

pub fn indirect_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("temp_indirect_buffer"),
        size,
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn get_atlas_image() -> image::RgbaImage {
    let q = std::env::current_exe().unwrap();
    let project_root = q
        .parent() // target/debug
        .unwrap()
        .parent() // target
        .unwrap()
        .parent() // project root
        .unwrap();

    let atlas_image = image::open(project_root.join("src/renderer/texture/images/atlas.png"))
        .expect("failed to load atlas.png");
    atlas_image.to_rgba8()
}

pub fn create_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render_encoder"),
    })
}
