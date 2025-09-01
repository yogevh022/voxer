use super::RendererBuilder;
use super::resources;
use crate::renderer::builder::layouts::create_texture_layout;
use image::RgbaImage;

pub struct RendererAtlas {
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl RendererBuilder<'_> {
    pub fn make_atlas(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        atlas_rgba: RgbaImage,
    ) -> RendererAtlas {
        let atlas_texture = resources::texture::create_diffuse(device, queue, &atlas_rgba);
        let texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = resources::texture::diffuse_sampler(device);
        let bind_group_entries = resources::utils::index_based_entries([
            wgpu::BindingResource::TextureView(&texture_view),
            wgpu::BindingResource::Sampler(&sampler),
        ]);

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_layout,
            entries: &bind_group_entries,
        });

        RendererAtlas {
            texture_view,
            sampler,
            bind_group: texture_bind_group,
        }
    }
}
