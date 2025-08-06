use crate::render::types::Mesh;
use crate::worldgen::types::block::BlockKind;
use std::time::Instant;

pub const CHUNK_SIZE: usize = 16;
pub type ChunkBlocks = [[[BlockKind; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

#[derive(Debug, Clone)]
pub struct Chunk {
    pub(crate) last_visited: Option<Instant>,
    pub(crate) blocks: ChunkBlocks,
    pub(crate) mesh: Option<Mesh>,
}
