mod alloc;
pub mod bind_group;
pub mod index;
pub mod pipeline;
pub mod render_pass;
mod resources;
pub mod shader;
pub mod texture;
pub mod uniform;
pub mod vertex;

use crate::render::renderer::resources::{
    MeshBuffers, RenderResources, TerrainResources, UniformResources,
};
use crate::render::types::{Index, Vertex};
use crate::types::SceneObject;
use crate::{render, types, utils};
use std::sync::Arc;
use winit::window::Window;

pub(crate) struct RendererState<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    pipeline: wgpu::RenderPipeline,
    resources: RenderResources,
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
        let atlas_texture = texture::create_diffuse(&device, &queue, &atlas_rgba);
        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = texture::diffuse_sampler(&device);
        let terrain_texture_bg_entries = bind_group::index_based_entries([
            // the index of the resource in this array is the index of the binding
            wgpu::BindingResource::TextureView(&atlas_texture_view),
            wgpu::BindingResource::Sampler(&atlas_sampler),
        ]);

        let uniform_buffer = uniform::create_buffer(
            &device,
            // arbitrarily creating the buffer with capacity for 1000 objects
            (device.limits().min_uniform_buffer_offset_alignment * 1000) as u64,
        );

        let uniform_buffer_binding_resource = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &uniform_buffer,
            offset: 0,
            size: std::num::NonZeroU64::new(64),
        });

        let (tex_bg_layout, tex_bg) =
            texture::create_bind_group(&device, &terrain_texture_bg_entries);
        let (uniform_bg_layout, uniform_bg) =
            uniform::create_bind_group(&device, uniform_buffer_binding_resource);

        let shader = shader::create(&device, shader::main_shader_source().into());
        let pipeline = pipeline::create(
            &device,
            &[&tex_bg_layout, &uniform_bg_layout],
            &shader,
            surface_format,
        );

        let (vertex_alloc, index_alloc) = alloc::size_of_meshes_from_sos(rendered_objects);
        let terrain_vertex_buffer = vertex::create_buffer(&device, vertex_alloc);
        let terrain_index_buffer = index::create_buffer(&device, index_alloc);

        let terrain_resources = TerrainResources {
            mesh_buffers: MeshBuffers {
                vertex: terrain_vertex_buffer,
                index: terrain_index_buffer,
            },
            texture_view: atlas_texture_view,
            texture_sampler: atlas_sampler,
            texture_bind_group: tex_bg,
        };

        let uniform_resources = UniformResources {
            buffer: uniform_buffer,
            bind_group: uniform_bg,
        };

        let depth_texture = texture::create_depth(&device, &config);
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let resources = RenderResources {
            terrain: terrain_resources,
            uniform: uniform_resources,
            depth_texture_view,
        };

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            resources,
        }
    }

    pub(crate) fn render(
        &mut self,
        camera: &types::Camera,
        scene: &mut types::Scene,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        let mut render_pass =
            render_pass::begin(&mut encoder, &view, &self.resources.depth_texture_view);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.resources.terrain.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.resources.terrain.mesh_buffers.vertex.slice(..));
        render_pass.set_index_buffer(
            self.resources.terrain.mesh_buffers.index.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        let uni_buf_offset = self.device.limits().min_uniform_buffer_offset_alignment;
        let mut global_mesh_data = render::types::global::Meshes::new();

        let vp = camera.get_view_projection();
        for (i, so) in scene.objects.iter_mut().enumerate() {
            let uni = (vp * so.model_matrix()).to_cols_array_2d();
            self.queue.write_buffer(
                &self.resources.uniform.buffer,
                i as u64 * uni_buf_offset as u64,
                bytemuck::cast_slice(&[uni]),
            );

            global_mesh_data.extend_with_offset(&so.model.mesh);
        }

        self.queue.write_buffer(
            &self.resources.terrain.mesh_buffers.vertex,
            0,
            bytemuck::cast_slice(&global_mesh_data.vertices),
        );
        self.queue.write_buffer(
            &self.resources.terrain.mesh_buffers.index,
            0,
            bytemuck::cast_slice(&global_mesh_data.indices),
        );

        let mut index_offset = 0u32; // in indices not bytes
        for (i, so) in scene.objects.iter().enumerate() {
            let buf_offset = i as u32 * uni_buf_offset;
            let index_count = so.model.mesh.indices.len() as u32;
            render_pass.set_bind_group(1, &self.resources.uniform.bind_group, &[buf_offset]);
            render_pass.draw_indexed(index_offset..(index_offset + index_count), 0, 0..1);
            index_offset += index_count;
        }
        drop(render_pass);

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}
