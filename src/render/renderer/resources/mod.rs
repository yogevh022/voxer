mod mesh;
mod terrain;
mod uniform;

pub use mesh::MeshBuffers;
pub use terrain::TerrainResources;
pub use uniform::UniformResources;

pub struct RenderResources {
    pub terrain: TerrainResources,
    pub uniform: UniformResources,
    pub depth_texture_view: wgpu::TextureView,
}
