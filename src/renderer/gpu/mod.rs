mod chunk_entry;
mod virtual_alloc;

pub use chunk_entry::{GPU_CHUNK_SIZE, GPUChunkEntry, GPUChunkEntryHeader};
pub use virtual_alloc::VirtualMemAlloc;
