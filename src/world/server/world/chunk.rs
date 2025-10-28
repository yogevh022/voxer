use glam::IVec3;
use std::array;
use crate::world::network::{MsgChunkData, MsgChunkDataEmpty};
use crate::world::server::world::{VoxelChunkAdjBlocks, VoxelChunkBlocks, CHUNK_DIM};
use crate::world::server::world::block::VoxelBlock;

#[derive(Debug, Clone)]
pub struct VoxelChunk {
    pub position: IVec3,
    pub blocks: VoxelChunkBlocks,
    pub voxel_count: u32,
}

impl VoxelChunk {
    pub fn new(position: IVec3, blocks: VoxelChunkBlocks, voxel_count: u32) -> Self {
        Self {
            position,
            blocks,
            voxel_count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.voxel_count == 0
    }

    pub(crate) fn blocks_as_adj(&self) -> VoxelChunkAdjBlocks {
        let adj = [
            self.mx_layer_blocks(),
            self.my_layer_blocks(),
            self.mz_layer_blocks(),
            self.px_layer_blocks(),
            self.py_layer_blocks(),
            self.pz_layer_blocks(),
        ];
        adj
    }

    fn mx_layer_blocks(&self) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
        self.blocks[0]
    }

    fn my_layer_blocks(&self) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
        array::from_fn(|i| self.blocks[i][0])
    }

    fn mz_layer_blocks(&self) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
        array::from_fn(|x| array::from_fn(|y| self.blocks[x][y][0]))
    }

    fn px_layer_blocks(&self) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
        self.blocks[CHUNK_DIM - 1]
    }

    fn py_layer_blocks(&self) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
        array::from_fn(|i| self.blocks[i][CHUNK_DIM - 1])
    }

    fn pz_layer_blocks(&self) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
        array::from_fn(|x| array::from_fn(|y| self.blocks[x][y][CHUNK_DIM - 1]))
    }
}

impl From<MsgChunkData> for VoxelChunk {
    fn from(msg: MsgChunkData) -> Self {
        Self::new(
            msg.position,
            msg.blocks,
            msg.voxel_count,
        )
    }
}

impl From<MsgChunkDataEmpty> for VoxelChunk {
    fn from(msg: MsgChunkDataEmpty) -> Self {
        Self::new(
            msg.position,
            VoxelChunkBlocks::default(),
            0,
        )
    }
}