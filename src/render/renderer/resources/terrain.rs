use crate::render::renderer::resources::MeshBuffers;

pub struct TerrainResources {
    pub atlas_view: wgpu::TextureView,
    pub atlas_sampler: wgpu::Sampler,
    pub atlas_bind_group: wgpu::BindGroup,
}
