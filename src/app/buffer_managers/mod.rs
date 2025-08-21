mod chunk_compute_manager;
mod chunk_render_manager;

pub use chunk_compute_manager::ChunkComputeManager;
pub use chunk_render_manager::ChunkRenderManager;

#[derive(Clone, Copy)]
pub struct ComputeInstruction {
    pub target_staging_buffer: usize,
    pub buffer_type: BufferType,
    pub byte_offset: usize,
    pub byte_length: usize,
}

#[derive(Clone, Copy)]
pub enum BufferType {
    Vertex,
    Index,
    MMat,
    Chunk,
}

#[derive(Default, Clone, Copy)]
pub struct MultiDrawInstruction {
    pub offset: usize,
    pub count: usize,
}
