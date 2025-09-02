use crate::world::types::{Block, CHUNK_DIM};
use glam::IVec3;

pub struct NetworkChunkSlice {
    pub slice_index: u8,
    pub position: IVec3,
    pub data: [[Block; CHUNK_DIM]; CHUNK_DIM],
}

impl NetworkChunkSlice {
    pub fn from_raw_data(data: &[u8]) -> Result<Self, ()> {
        if data.len() != size_of::<Self>() {
            return Err(());
        }
        let slice_index: &u8 = bytemuck::try_from_bytes(&data[0..1]).ok().ok_or(())?;
        let position: &IVec3 = bytemuck::try_from_bytes(&data[1..size_of::<IVec3>() + 1])
            .ok()
            .ok_or(())?;
        let data: &[[Block; CHUNK_DIM]; CHUNK_DIM] =
            bytemuck::try_from_bytes(&data[size_of::<IVec3>() + 1..])
                .ok()
                .ok_or(())?;
        Ok(Self {
            slice_index: *slice_index,
            position: *position,
            data: *data,
        })
    }
}

pub enum ServerMessage {
    ChunkSlice(NetworkChunkSlice),
}

impl TryFrom<Vec<u8>> for ServerMessage {
    type Error = ();
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value[0] {
            0 => Ok(ServerMessage::ChunkSlice(NetworkChunkSlice::from_raw_data(&value[1..])?)),
            _ => Err(()),
        }
    }
}
