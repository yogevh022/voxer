pub mod bind_group;
pub mod index;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod texture;
pub mod uniform;
pub mod vertex;

use crate::render::types::Vertex;
use crate::types::SceneObject;
use crate::{render, types, utils};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

pub(crate) struct RendererState<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,

    depth_texture: wgpu::Texture,
    atlas_texture: wgpu::Texture,
    atlas_texture_view: wgpu::TextureView,
    atlas_sampler: wgpu::Sampler,

    texture_bind_group: wgpu::BindGroup,
    uniform_bind_group: wgpu::BindGroup,
}

impl RendererState<'_> {
    pub(crate) async fn new(window: Arc<Window>, rendered_objects: &Vec<SceneObject>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .await
            .unwrap();

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let atlas_rgba = utils::temp::get_atlas_image();

        let depth_texture = texture::create_depth(&device, &config);
        let atlas_texture = texture::create_diffuse(&device, &queue, &atlas_rgba);
        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = texture::diffuse_sampler(&device);

        let uniform_buffer = uniform::create_buffer(
            &device,
            // arbitrarily creating the buffer with capacity for 1000 objects
            (device.limits().min_uniform_buffer_offset_alignment * 1000) as u64,
        );
        let mut vert_alloc = 0u64;
        let mut ind_alloc = 0u64;
        rendered_objects.iter().for_each(|so| {
            vert_alloc += so.model.mesh.vertex_offset;
            ind_alloc += so.model.mesh.index_offset;
        });

        let vertex_buffer = vertex::create_buffer(&device, vert_alloc * size_of::<Vertex>() as u64);
        let index_buffer = index::create_buffer(&device, ind_alloc * size_of::<u16>() as u64);

        let tex_bg_entries = bind_group::index_based_entries([
            // the index of the resource in this array is the index of the binding
            wgpu::BindingResource::TextureView(&atlas_texture_view),
            wgpu::BindingResource::Sampler(&atlas_sampler),
        ]);

        let uniform_buffer_binding_resource = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &uniform_buffer,
            offset: 0,
            size: std::num::NonZeroU64::new(64),
        });

        let (tex_bg_layout, tex_bg) = texture::create_bind_group(&device, &tex_bg_entries);
        let (uniform_bg_layout, uniform_bg) =
            uniform::create_bind_group(&device, uniform_buffer_binding_resource);

        let shader = shader::create(&device, shader::main_shader_source().into());
        let render_pipeline = pipeline::create(
            &device,
            &[&tex_bg_layout, &uniform_bg_layout],
            &shader,
            surface_format,
        );

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            depth_texture,
            atlas_texture,
            atlas_texture_view,
            atlas_sampler,
            size,
            texture_bind_group: tex_bg,
            uniform_bind_group: uniform_bg,
        }
    }

    pub(crate) fn update_vertex_buffer(&mut self, vertices: &[Vertex]) {
        // fixme
        self.vertex_buffer = vertex::create_buffer(&self.device, vertices.len() as u64);
    }

    pub(crate) fn render(
        &mut self,
        camera: &types::Camera,
        scene: &mut types::Scene,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let texture_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        let mut render_pass = render_pass::begin(&mut encoder, &texture_view, &depth_view);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        let uni_buf_offset = self.device.limits().min_uniform_buffer_offset_alignment;
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let vp = camera.get_view_projection();

        for (i, so) in scene.objects.iter_mut().enumerate() {
            let uni = (vp * so.model_matrix()).to_cols_array_2d();
            self.queue.write_buffer(
                &self.uniform_buffer,
                i as u64 * uni_buf_offset as u64,
                bytemuck::cast_slice(&[uni]),
            );

            all_vertices.extend_from_slice(&so.model.mesh.vertices);
            all_indices.extend_from_slice(&so.model.mesh.indices);
        }

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&all_indices));

        let mut index_offset = 0u32; // in indices not bytes
        for (i, so) in scene.objects.iter().enumerate() {
            let buf_offset = i as u32 * uni_buf_offset;
            let index_count = so.model.mesh.indices.len() as u32;
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[buf_offset]);
            render_pass.draw_indexed(index_offset..(index_offset + index_count), 0, 0..1);
            index_offset += index_count;
        }
        drop(render_pass);

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}
