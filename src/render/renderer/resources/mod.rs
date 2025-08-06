mod mesh;
mod terrain;
mod uniform;

use crate::render::renderer::core::ChunkBufferEntry;
pub use mesh::MeshBuffers;
pub use terrain::TerrainResources;
pub use uniform::UniformResources;

pub struct RenderResources {
    pub terrain: TerrainResources,
    pub uniform: UniformResources,
    pub depth_texture_view: wgpu::TextureView,
    pub chunk_buffer_pool: Vec<ChunkBufferEntry>,
}
