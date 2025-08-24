mod chunk_compute_manager;
mod chunk_render_manager;

pub use chunk_compute_manager::ChunkComputeManager;
pub use chunk_render_manager::ChunkRenderManager;

#[derive(Clone, Copy, Debug)]
pub struct ComputeInstruction {
    pub target_buffer: usize,
    pub vertex_offset_bytes: u64,
    pub index_offset_bytes: u64,
    pub mmat_offset_bytes: u64,
    pub vertex_size_bytes: u64,
    pub index_size_bytes: u64,
    pub mmat_size_bytes: u64,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct WriteInstruction<'a> {
    pub staging_index: usize,
    pub bytes: &'a [u8],
    pub offset: u64,
}

#[derive(Default, Clone, Copy)]
#[derive(Debug)]
pub struct MultiDrawInstruction {
    pub offset: usize,
    pub count: usize,
}
