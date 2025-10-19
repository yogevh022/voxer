use image::RgbaImage;
use wgpu::{BindGroup, BindGroupLayout, TextureView};
use crate::renderer::{resources, Renderer};

impl Renderer<'_> {
    pub fn texture_sampler(
        &self,
        label: &str,
        rgba_image: RgbaImage,
    ) -> (BindGroupLayout, BindGroup) {
        let texture = create_diffuse(&self.device, &self.queue, &rgba_image);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = diffuse_sampler(&self.device);
        let bind_group_entries = resources::utils::bind_entries([
            wgpu::BindingResource::TextureView(&texture_view),
            wgpu::BindingResource::Sampler(&sampler),
        ]);
        let texture_sampler_layout = texture_sampler_bind_layout(&self.device);
        let texture_sampler_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &texture_sampler_layout,
            entries: &bind_group_entries,
        });
        (texture_sampler_layout, texture_sampler_bind_group)
    }

    pub fn dbg_sampler(&self, tempv: &TextureView) -> (BindGroupLayout, BindGroup) {
        let temps = self.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let debug_bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("DEBUG Sampler Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let bg_entries = resources::utils::bind_entries([
            wgpu::BindingResource::TextureView(tempv),
            wgpu::BindingResource::Sampler(&temps),
        ]);

        let debug_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &debug_bgl,
            entries: &bg_entries,
            label: Some("debug depth mip bind group"),
        });
        (debug_bgl, debug_bg)
    }
}


pub fn create_diffuse(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &image::RgbaImage,
) -> wgpu::Texture {
    // creates and writes texture to device
    let texture_size = get_image_extent3d(image);
    let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("diffuse_texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &image,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * texture_size.width),
            rows_per_image: Some(texture_size.height),
        },
        texture_size,
    );
    diffuse_texture
}

pub fn diffuse_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    // preconfigured sampler
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

fn texture_sampler_bind_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Sampler Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

#[inline]
pub fn get_image_extent3d(image: &image::RgbaImage) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: image.width(),
        height: image.height(),
        depth_or_array_layers: 1,
    }
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