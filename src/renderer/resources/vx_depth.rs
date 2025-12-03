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
    pub mip_texture_array: Texture,
    pub mip_texture_array_view: TextureView,
    pub mip_views: Box<[TextureView]>,
    mip_bind_groups: Box<[BindGroup]>,
    mip_one_compute_bgl: BindGroupLayout,
    mip_one_compute_pipeline: ComputePipeline,
    mip_x_compute_bgl: BindGroupLayout,
    mip_x_compute_pipeline: ComputePipeline,
}

impl VxDepth {
    pub fn new(device: &Device, size: PhysicalSize<u32>) -> Self {
        let mip_count = (size.width.max(size.height) as f32).log2().floor() as u32 + 1;
        let mut surface_extent = Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let texture = create_depth_texture(device, surface_extent);
        let view = texture.create_view(&Default::default());

        surface_extent.depth_or_array_layers = mip_count;
        let mip_texture_array = create_depth_mip_texture_array(device, surface_extent);
        let mip_texture_array_view = Self::mip_array_view(&mip_texture_array, mip_count);
        let mip_views = Self::mip_views(&mip_texture_array, mip_count);

        let mip_one_compute_bgl = depth_mip_one_bgl(device);
        let mip_one_compute_pipeline = depth_mip_one_pipeline(device, &[&mip_one_compute_bgl]);
        let mip_x_compute_bgl = depth_mip_x_bgl(device);
        let mip_x_compute_pipeline = depth_mip_x_pipeline(device, &[&mip_x_compute_bgl]);

        let mip_bind_groups = Self::mip_bind_groups(
            device,
            &mip_one_compute_bgl,
            &mip_x_compute_bgl,
            &view,
            &mip_views,
            mip_count as usize,
        );

        Self {
            surface_extent,
            texture,
            view,
            mip_texture_array,
            mip_texture_array_view,
            mip_views,
            mip_bind_groups,
            mip_one_compute_bgl,
            mip_one_compute_pipeline,
            mip_x_compute_bgl,
            mip_x_compute_pipeline,
        }
    }

    fn mip_array_view(mip_texture_array: &Texture, mip_count: u32) -> TextureView {
        mip_texture_array.create_view(&TextureViewDescriptor {
            label: Some("Depth Mip Array View"),
            base_array_layer: 0,
            array_layer_count: Some(mip_count),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        })
    }

    fn mip_views(texture: &Texture, mip_count: u32) -> Box<[TextureView]> {
        let mut mip_views = Vec::with_capacity(mip_count as usize);
        for mip in 0..mip_count {
            let view = texture.create_view(&TextureViewDescriptor {
                label: Some(&format!("Depth Mip View [{}]", mip)),
                base_array_layer: mip,
                array_layer_count: Some(1),
                dimension: Some(TextureViewDimension::D2Array),
                ..Default::default()
            });
            mip_views.push(view);
        }
        mip_views.into_boxed_slice()
    }

    fn mip_bind_group(
        device: &Device,
        bgl: &BindGroupLayout,
        src: &TextureView,
        dst: &TextureView,
    ) -> BindGroup {
        let entries = [
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(src),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(dst),
            },
        ];
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: bgl,
            entries: &entries,
        })
    }

    fn mip_bind_groups(
        device: &Device,
        one_bgl: &BindGroupLayout,
        x_bgl: &BindGroupLayout,
        depth_view: &TextureView,
        mip_depth_views: &[TextureView],
        mip_count: usize,
    ) -> Box<[BindGroup]> {
        let mut mip_bind_groups = Vec::with_capacity(mip_count);
        mip_bind_groups.push(Self::mip_bind_group(
            device,
            one_bgl,
            depth_view,
            &mip_depth_views[1],
        ));
        for mip in 2..mip_count - 1 {
            mip_bind_groups.push(Self::mip_bind_group(
                device,
                x_bgl,
                &mip_depth_views[mip - 1],
                &mip_depth_views[mip],
            ));
        }
        mip_bind_groups.into_boxed_slice()
    }

    fn generate_depth_mip_one(&self, compute_pass: &mut ComputePass) {
        compute_pass.set_pipeline(&self.mip_one_compute_pipeline);
        compute_pass.set_bind_group(0, &self.mip_bind_groups[0], &[]);
        let mip_width = self.surface_extent.width >> 1;
        let mip_height = self.surface_extent.height >> 1;
        let groups_x = ceil_div(mip_width, MAX_WORKGROUP_DIM_2D);
        let groups_y = ceil_div(mip_height, MAX_WORKGROUP_DIM_2D);
        compute_pass.set_push_constants(0, bytemuck::cast_slice(&[mip_width, mip_height]));
        compute_pass.dispatch_workgroups(groups_x, groups_y, 1);
    }

    pub fn generate_depth_mips(&self, compute_pass: &mut ComputePass) {
        self.generate_depth_mip_one(compute_pass);
        compute_pass.set_pipeline(&self.mip_x_compute_pipeline);
        for mip in 1..self.mip_bind_groups.len() {
            // last mip unused
            let mip_bg = &self.mip_bind_groups[mip];
            compute_pass.set_bind_group(0, mip_bg, &[]);
            let mip_width = (self.surface_extent.width >> mip).max(1);
            let mip_height = (self.surface_extent.height >> mip).max(1);
            let groups_x = ceil_div(mip_width, MAX_WORKGROUP_DIM_2D);
            let groups_y = ceil_div(mip_height, MAX_WORKGROUP_DIM_2D);
            compute_pass.set_push_constants(0, bytemuck::cast_slice(&[mip_width, mip_height]));
            compute_pass.dispatch_workgroups(groups_x, groups_y, 1);
        }
    }
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

fn create_depth_mip_texture_array(device: &Device, surface_extent: Extent3d) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Depth Mip Texture Array"),
        size: surface_extent,
        mip_level_count: 1,
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

fn depth_mip_one_bgl(device: &Device) -> BindGroupLayout {
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
                    view_dimension: TextureViewDimension::D2Array,
                },
                count: None,
            },
        ],
    })
}

fn depth_mip_x_bgl(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Depth Mip X Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0, // src
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadOnly,
                    format: TextureFormat::R32Float,
                    view_dimension: TextureViewDimension::D2Array,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1, // dst
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::R32Float,
                    view_dimension: TextureViewDimension::D2Array,
                },
                count: None,
            },
        ],
    })
}

fn depth_mip_one_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create_shader(
        device,
        resources::shader::depth_mip_one_wgsl().into(),
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
        entry_point: Some("depth_mip_one_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn depth_mip_x_pipeline(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let shader = resources::shader::create_shader(
        device,
        resources::shader::depth_mip_x_wgsl().into(),
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
        entry_point: Some("depth_mip_x_entry"),
        compilation_options: Default::default(),
        cache: None,
    })
}
