mod chunk;
mod mesh;
mod terrain;
mod uniform;

pub use chunk::{ChunkPool, ChunkPoolEntry};
pub use mesh::MeshBuffers;
pub use terrain::TerrainResources;
pub use uniform::TransformResources;
use wgpu::naga::FastHashMap;

pub struct RenderResources {
    pub terrain: TerrainResources,
    pub transform: TransformResources,
    pub chunk_pool: ChunkPool,
    pub depth_texture_view: wgpu::TextureView,
}

pub struct ComputeResources {
    pub pipelines: FastHashMap<&'static str, wgpu::ComputePipeline>,
    pub bind_groups: FastHashMap<&'static str, wgpu::BindGroup>,
}
