use super::{Renderer, resources};
use std::sync::Arc;
use winit::window::Window;

mod atlas;
mod buffer;
mod layouts;
mod render_pipeline;

use crate::compute;
use crate::renderer::builder::layouts::{
    create_mmat_layout, create_texture_layout, create_view_projection_layout,
};
use crate::renderer::resources::texture::get_atlas_image;

pub struct RendererBuilder<'window> {
    pub(crate) config: Option<wgpu::SurfaceConfiguration>,
    pub(crate) surface: Option<wgpu::Surface<'window>>,
    pub(crate) surface_format: Option<wgpu::TextureFormat>,
    pub(crate) device: Option<wgpu::Device>,
    pub(crate) queue: Option<wgpu::Queue>,
    pub(crate) indirect_buffer: Option<wgpu::Buffer>,
    pub(crate) view_projection_buffer: Option<wgpu::Buffer>,
    pub(crate) depth_texture_view: Option<wgpu::TextureView>,
    pub(crate) mmat_layout: Option<wgpu::BindGroupLayout>,
    pub(crate) view_projection_layout: Option<wgpu::BindGroupLayout>,
    pub(crate) texture_atlas_layout: Option<wgpu::BindGroupLayout>,
    pub(crate) texture_atlas_bind_group: Option<wgpu::BindGroup>,
    pub(crate) view_projection_bind_group: Option<wgpu::BindGroup>,
}

impl<'window> RendererBuilder<'window> {
    pub(crate) fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE
                | wgpu::Features::INDIRECT_FIRST_INSTANCE
                | wgpu::Features::MULTI_DRAW_INDIRECT,
            required_limits: wgpu::Limits::default(),
            label: None,
            memory_hints: Default::default(),
            trace: Default::default(),
        }))
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

        let indirect_buffer = RendererBuilder::make_buffer(
            &device,
            "temp_indirect_buffer",
            250 * compute::KIB as u64,
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        );

        let uniform_buffer_size = device
            .limits()
            .min_uniform_buffer_offset_alignment
            .min(size_of::<[[f32; 4]; 4]>() as u32) as u64;
        let view_projection_buffer = RendererBuilder::make_buffer(
            &device,
            "view_projection_buffer",
            uniform_buffer_size,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        let depth_texture_view = resources::texture::create_depth(&device, &config)
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mmat_layout = create_mmat_layout(&device);
        let view_projection_layout = create_view_projection_layout(&device);
        let texture_atlas_layout = create_texture_layout(&device);

        let view_projection_binding = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &view_projection_buffer,
            offset: 0,
            size: std::num::NonZeroU64::new(view_projection_buffer.size()),
        });

        let view_projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &view_projection_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_projection_binding,
            }],
            label: Some("view_projection_bind_group"),
        });

        let texture_atlas_bind_group =
            RendererBuilder::make_atlas(&device, &queue, &texture_atlas_layout, get_atlas_image())
                .bind_group;

        Self {
            surface: Some(surface),
            surface_format: Some(surface_format),
            config: Some(config),
            device: Some(device),
            queue: Some(queue),
            indirect_buffer: Some(indirect_buffer),
            view_projection_buffer: Some(view_projection_buffer),
            depth_texture_view: Some(depth_texture_view),
            mmat_layout: Some(mmat_layout),
            view_projection_layout: Some(view_projection_layout),
            texture_atlas_layout: Some(texture_atlas_layout),
            texture_atlas_bind_group: Some(texture_atlas_bind_group),
            view_projection_bind_group: Some(view_projection_bind_group),
        }
    }

    pub fn build(self) -> Renderer<'window> {
        Renderer::from(self)
    }
}
