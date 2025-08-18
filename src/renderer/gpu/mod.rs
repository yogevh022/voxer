mod chunk_entry;
mod virtual_alloc;

pub use chunk_entry::{GPUChunkEntry, GPUChunkEntryBuffer, GPUChunkEntryHeader};
pub use virtual_alloc::{ChunkVMA, VirtualMemAlloc};
