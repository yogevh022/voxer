use super::{Renderer, resources};
use std::sync::Arc;
use winit::window::Window;

mod atlas;
mod buffer;
mod render_pipeline;
pub use atlas::RendererAtlas;

pub struct RendererBuilder<'window> {
    pub(crate) config: Option<wgpu::SurfaceConfiguration>,
    pub(crate) surface: Option<wgpu::Surface<'window>>,
    pub(crate) surface_format: Option<wgpu::TextureFormat>,
    pub(crate) device: Option<wgpu::Device>,
    pub(crate) queue: Option<wgpu::Queue>,
    pub(crate) view_projection_buffer: Option<wgpu::Buffer>,
    pub(crate) depth_texture_view: Option<wgpu::TextureView>,
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

        let uniform_buffer_size = device
            .limits()
            .min_uniform_buffer_offset_alignment
            .min(size_of::<[[f32; 4]; 4]>() as u32) as u64;
        let view_projection_buffer = resources::uniform::create_buffer(
            &device,
            uniform_buffer_size,
            "view_projection_buffer",
        );

        let depth_texture_view = resources::texture::create_depth(&device, &config)
            .create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            surface: Some(surface),
            surface_format: Some(surface_format),
            device: Some(device),
            queue: Some(queue),
            config: Some(config),
            view_projection_buffer: Some(view_projection_buffer),
            depth_texture_view: Some(depth_texture_view),
        }
    }

    pub fn build(self) -> Renderer<'window> {
        Renderer::from(self)
    }
}
