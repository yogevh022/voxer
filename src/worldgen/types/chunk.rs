use crate::worldgen::types::block::BlockKind;

pub struct Chunk {
    pub(crate) blocks: [[[BlockKind; 16]; 16]; 16],
}
