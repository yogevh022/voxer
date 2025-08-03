mod app;
mod constants;
mod mat;
mod render;
mod texture;
mod types;
mod utils;

use crate::mat::model_to_world_matrix;
use crate::render::types::Vertex;
use crate::texture::{Texture, TextureAtlas};
use glam::{Mat4, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::event::*;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    }, // 0
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    }, // 1
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    }, // 2
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    }, // 3
];

const INDICES: &[u16] = &[0, 2, 1, 0, 3, 2];

const VERT_SHADER: &str = include_str!("shaders/vert.wgsl");
const FRAG_SHADER: &str = include_str!("shaders/frag.wgsl");

struct RendererState<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    num_vertices: u32,
    num_indices: u32,

    atlas_texture: wgpu::Texture,
    atlas_texture_view: wgpu::TextureView,
    atlas_sampler: wgpu::Sampler,

    texture_bind_group: wgpu::BindGroup,
    uniform_bind_group: wgpu::BindGroup,
}

impl RendererState<'_> {
    async fn new(window: Arc<Window>) -> Self {
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

        let atlas_texture = render::texture::create_diffuse(&device, &queue, &atlas_rgba);
        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = render::texture::diffuse_sampler(&device);

        let uniform_buffer = render::uniform::create_buffer(&device);
        let vertex_buffer = render::vertex::create_buffer(&device, VERTICES);
        let index_buffer = render::index::create_buffer(&device, INDICES);

        let bg_entries = render::bind_group::index_based_entries([
            // the index of the resource in this array is the index of the binding
            wgpu::BindingResource::TextureView(&atlas_texture_view),
            wgpu::BindingResource::Sampler(&atlas_sampler),
        ]);

        let (tex_bg_layout, tex_bg) = render::texture::create_bind_group(&device, &bg_entries);
        let (uniform_bg_layout, uniform_bg) =
            render::uniform::create_bind_group(&device, uniform_buffer.as_entire_binding());

        let shader = render::shader::create(&device, render::shader::main_shader_source().into());
        let render_pipeline = render::pipeline::create(
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
            num_vertices: VERTICES.len() as u32,
            num_indices: INDICES.len() as u32,
            atlas_texture,
            atlas_texture_view,
            atlas_sampler,
            size,
            texture_bind_group: tex_bg,
            uniform_bind_group: uniform_bg,
        }
    }

    fn update_vertex_buffer(&mut self, vertices: &[Vertex]) {
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.num_vertices = vertices.len() as u32;
    }

    fn render(
        &mut self,
        camera: &types::Camera,
        scene: &types::Scene,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let model = model_to_world_matrix(Vec3::new(0f32, 0f32, -5f32), 0f32, 2f32);
        let view_m = Mat4::look_to_rh(
            camera.transform.position,
            camera.target,
            camera.transform.up(),
        );
        let proj = Mat4::perspective_rh(
            camera.properties.fov,
            camera.properties.aspect_ratio,
            camera.properties.near,
            camera.properties.far,
        );

        let uniform = (proj * view_m * model).to_cols_array_2d();
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&uniform));

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let scene = types::Scene::default();
    let camera = types::Camera {
        target: Vec3::new(0.0, 0.0, -1.0),
        ..Default::default()
    };

    let mut app = app::App {
        window: None,
        state: None,
        scene,
        camera,
    };

    event_loop.run_app(&mut app).unwrap();
}

// for object in &scene.objects {
//     let mvp = projection * view * object.model_matrix;
//     queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[mvp]));
//     render_pass.set_bind_group(0, &uniform_bind_group, &[]);
//     render_pass.draw_indexed(object.index_range.clone(), 0, 0..1);
// }
