use super::RendererBuilder;
use super::resources;
use image::RgbaImage;

pub struct RendererAtlas {
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl RendererBuilder<'_> {
    pub fn make_atlas(&self, atlas_rgba: RgbaImage) -> RendererAtlas {
        let atlas_texture = resources::texture::create_diffuse(
            self.device.as_ref().unwrap(),
            self.queue.as_ref().unwrap(),
            &atlas_rgba,
        );
        let texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = resources::texture::diffuse_sampler(self.device.as_ref().unwrap());
        let bind_group_entries = resources::bind_group::index_based_entries([
            wgpu::BindingResource::TextureView(&texture_view),
            wgpu::BindingResource::Sampler(&sampler),
        ]);

        let (texture_bind_group_layout, bind_group) = resources::texture::create_bind_group(
            self.device.as_ref().unwrap(),
            &bind_group_entries,
        );

        RendererAtlas {
            texture_view,
            sampler,
            bind_group,
            texture_bind_group_layout,
        }
    }
}
