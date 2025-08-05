use crate::render::renderer::resources::MeshBuffers;

pub struct TerrainResources {
    pub mesh_buffers: MeshBuffers,
    pub texture_view: wgpu::TextureView,
    pub texture_sampler: wgpu::Sampler,
    pub texture_bind_group: wgpu::BindGroup,
}
