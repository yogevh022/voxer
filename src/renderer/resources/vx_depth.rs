use crate::compute::num::ceil_div;
use crate::renderer::resources;
use crate::renderer::resources::shader::MAX_WORKGROUP_DIM_2D;
use wgpu::wgt::TextureDescriptor;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, Device, Extent3d, PipelineLayoutDescriptor, PushConstantRange,
    ShaderStages, StorageTextureAccess, Texture, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};
use winit::dpi::PhysicalSize;

pub struct VxDepth {
    surface_extent: Extent3d,
    pub texture: Texture,
    pub view: TextureView,
    pub mip_texture: Texture,
    pub mip_views: Box<[TextureView]>,
    mip1_compute_bgl: BindGroupLayout,
    mip1_compute_pipeline: ComputePipeline,
    mipx_compute_bgl: BindGroupLayout,
    mipx_compute_pipeline: ComputePipeline,
}

impl VxDepth {
    pub fn new(device: &Device, size: PhysicalSize<u32>) -> Self {
        let depth_mip_count = (size.width.max(size.height) as f32).log2().floor() as u32 + 1;
        let surface_extent = Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let texture = create_depth_texture(device, surface_extent);
        let view = texture.create_view(&Default::default());

        let mip_texture = create_depth_mip_texture(device, surface_extent, depth_mip_count);
        let mip_views = mip_views(&mip_texture, depth_mip_count);

        let mip1_compute_bgl = depth_mip1_bgl(device);
        let mip1_compute_pipeline = depth_mip1_pipeline(device, &[&mip1_compute_bgl]);
        let mipx_compute_bgl = depth_mipx_bgl(device);
        let mipx_compute_pipeline = depth_mipx_pipeline(device, &[&mipx_compute_bgl]);

        Self {
            surface_extent,
            texture,
            view,
            mip_texture,
            mip_views,
            mip1_compute_bgl,
            mip1_compute_pipeline,
            mipx_compute_bgl,
            mipx_compute_pipeline,
        }
    }

    fn mip1_bind_group(&self, device: &Device) -> BindGroup {
        let entries = [
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&self.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&self.mip_views[1]),
            },
        ];
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.mip1_compute_bgl,
            entries: &entries,
        })
    }

    fn mipx_bind_group(&self, device: &Device, src_mip: usize, dst_mip: usize) -> BindGroup {
        let entries = [
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&self.mip_views[src_mip]),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&self.mip_views[dst_mip]),
            },
        ];
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.mipx_compute_bgl,
            entries: &entries,
        })
    }

    pub fn generate_initial_mip(&self, device: &Device, compute_pass: &mut ComputePass) {
        compute_pass.set_pipeline(&self.mip1_compute_pipeline);
        compute_pass.set_bind_group(0, &self.mip1_bind_group(device), &[]);
        let mip_width = self.surface_extent.width >> 1;
        let mip_height = self.surface_extent.height >> 1;
        let groups_x = ceil_div(mip_width, MAX_WORKGROUP_DIM_2D);
        let groups_y = ceil_div(mip_height, MAX_WORKGROUP_DIM_2D);
        compute_pass.set_push_constants(0, bytemuck::cast_slice(&[mip_width, mip_height]));
        compute_pass.dispatch_workgroups(groups_x, groups_y, 1);
    }

    pub fn generate_depth_mips(&self, device: &Device, compute_pass: &mut ComputePass) {
        compute_pass.set_pipeline(&self.mipx_compute_pipeline);
        for mip in 2..self.mip_views.len() {
            let mip_bg = self.mipx_bind_group(device, mip - 1, mip);
            compute_pass.set_bind_group(0, &mip_bg, &[]);
            let mip_width = (self.surface_extent.width >> mip).max(1);
            let mip_height = (self.surface_extent.height >> mip).max(1);
            let groups_x = ceil_div(mip_width, MAX_WORKGROUP_DIM_2D);
            let groups_y = ceil_div(mip_height, MAX_WORKGROUP_DIM_2D);
            compute_pass.set_push_constants(0, bytemuck::cast_slice(&[mip_width, mip_height]));
            compute_pass.dispatch_workgroups(groups_x, groups_y, 1);
        }
    }
}

fn mip_views(mip_texture: &Texture, mip_count: u32) -> Box<[TextureView]> {
    let mut mip_views = Vec::with_capacity(mip_count as usize);
    for mip in 0..mip_count {
        let view = mip_texture.create_view(&TextureViewDescriptor {
            label: Some(&format!("Depth Mip View [{}]", mip)),
            base_mip_level: mip,
            mip_level_count: Some(1),
            ..Default::default()
        });
        mip_views.push(view);
    }
    mip_views.into_boxed_slice()
}

fn create_depth_texture(device: &Device, surface_extent: Extent3d) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Depth Texture"),
        size: surface_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn create_depth_mip_texture(
    device: &Device,
    surface_extent: Extent3d,
    depth_mip_count: u32,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Depth Mip Texture"),
        size: surface_extent,
        mip_level_count: depth_mip_count,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::R32Float,
        usage: TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_SRC
            | TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn depth_mip1_bgl(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Depth Mip 1 Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0, // src depth
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Depth,
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1, // dst
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::R32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    })
}

fn depth_mipx_bgl(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Depth Mip X Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0, // src
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadOnly,
                    format: TextureFormat::R32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1, // dst
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::R32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    })
}

fn depth_mip1_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create_shader(
        device,
        resources::shader::write_depth_mip1_wgsl().into(),
        "Depth Mip 1 Pipeline Shader",
    );
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Depth Mip 1 Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStages::COMPUTE,
            range: 0..8,
        }],
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Depth Mip 1 Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("depth_mip1_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn depth_mipx_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create_shader(
        device,
        resources::shader::write_depth_mipx_wgsl().into(),
        "Depth Mip X Pipeline Shader",
    );
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Depth Mip X Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStages::COMPUTE,
            range: 0..8,
        }],
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Depth Mip X Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("depth_mipx_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}
