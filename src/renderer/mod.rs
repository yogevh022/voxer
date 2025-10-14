pub mod gpu;
pub mod resources;
mod texture;
mod types;

use crate::compute;
use crate::renderer::gpu::GPU4Bytes;
use crate::renderer::resources::vx_buffer::VxBuffer;
use crate::renderer::resources::vx_device::VxDevice;
use crate::renderer::resources::vx_queue::VxQueue;
use glam::Mat4;
use std::sync::Arc;
pub use types::*;
use voxer_macros::ShaderType;
use wgpu::{
    Adapter, Backends, BufferAddress, BufferUsages, Device, Features, Instance, Limits, Queue,
    Surface, SurfaceCapabilities, SurfaceConfiguration, TextureView,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[derive(ShaderType)]
struct Shader20Bytes {
    bytes: [u8; 20],
}

pub(crate) struct Renderer<'window> {
    pub(crate) surface: Surface<'window>,
    pub(crate) adapter: Adapter,
    pub(crate) device: VxDevice,
    pub(crate) queue: VxQueue,
    pub(crate) indirect_buffer: VxBuffer,
    pub(crate) indirect_count_buffer: VxBuffer,
    pub(crate) surface_format: wgpu::TextureFormat,
    depth_texture_view: TextureView,
}

impl<'window> Renderer<'window> {
    const DXC_DLL_PATH: Option<&'static str> = Some(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/renderer/resources/dxc/bin/x64/dxcompiler.dll"
    ));
    fn instance(backends: Backends) -> Instance {
        Instance::new(&wgpu::InstanceDescriptor {
            backends,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: wgpu::BackendOptions {
                gl: Default::default(),
                dx12: wgpu::Dx12BackendOptions {
                    shader_compiler: wgpu::Dx12Compiler::DynamicDxc {
                        dxc_path: Self::DXC_DLL_PATH.unwrap_or_default().to_string(),
                        max_shader_model: wgpu::DxcShaderModel::V6_0,
                    },
                    presentation_system: Default::default(),
                    latency_waitable_object: Default::default(),
                },
                noop: Default::default(),
            },
        })
    }

    fn high_perf_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        }))
        .unwrap()
    }

    fn request_device(
        adapter: &Adapter,
        required_features: Features,
        required_limits: Limits,
    ) -> (Device, Queue) {
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features,
            required_limits,
            label: None,
            memory_hints: Default::default(),
            trace: Default::default(),
            experimental_features: Default::default(),
        }))
        .unwrap()
    }

    fn surface_config(
        surface_capabilities: &SurfaceCapabilities,
        size: PhysicalSize<u32>,
    ) -> SurfaceConfiguration {
        SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_capabilities.formats[0],
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        }
    }

    pub fn new(window: Arc<Window>) -> Self {
        let instance = Renderer::instance(Backends::DX12); // fixme dx12?
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = Renderer::high_perf_adapter(&instance, &surface);
        let (device, queue) = Renderer::request_device(
            &adapter,
            Features::VERTEX_WRITABLE_STORAGE
                | Features::INDIRECT_FIRST_INSTANCE
                | Features::MULTI_DRAW_INDIRECT_COUNT,
            Limits {
                max_storage_buffer_binding_size: (compute::GIB * 2) as u32 - 1,
                max_buffer_size: (compute::GIB as u64 * 2) - 1,
                ..Default::default()
            },
        );
        let vx_device = VxDevice::new(device);
        let vx_queue = VxQueue::new(queue);

        let size = window.inner_size();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_config = Renderer::surface_config(&surface_capabilities, size);
        surface.configure(&vx_device, &surface_config);

        let indirect_buffer = vx_device.create_vx_buffer::<Shader20Bytes>(
            "Indirect Buffer",
            (256 * compute::KIB as u64).try_into().unwrap(),
            BufferUsages::INDIRECT | BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let indirect_count_buffer = vx_device.create_vx_buffer::<GPU4Bytes>(
            "Indirect Count Buffer",
            1,
            BufferUsages::INDIRECT | BufferUsages::STORAGE | BufferUsages::COPY_DST,
        );

        let depth_texture = resources::texture::create_depth(&vx_device, &surface_config);
        let depth_texture_view = depth_texture.create_view(&Default::default());

        Self {
            surface,
            adapter,
            device: vx_device,
            queue: vx_queue,
            indirect_buffer,
            indirect_count_buffer,
            depth_texture_view,
            surface_format: surface_capabilities.formats[0],
        }
    }

    pub fn write_buffer(&self, buffer: &wgpu::Buffer, offset: BufferAddress, data: &[u8]) {
        self.queue.write_buffer(buffer, offset, data)
    }

    pub fn create_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }

    #[inline]
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn begin_render_pass<'e>(
        &self,
        encoder: &'e mut wgpu::CommandEncoder,
        label: &str,
        frame_view: &TextureView,
    ) -> wgpu::RenderPass<'e> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}
