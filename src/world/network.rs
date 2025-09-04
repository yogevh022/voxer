use crate::impl_try_from_uint;
use crate::voxer_network::{NetworkMessageConfig, NetworkMessageTag, ReceivedMessage};
use crate::world::types::{CHUNK_DIM, VoxelBlock};
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, Vec3};

#[derive(Debug)]
pub struct ServerMessage {
    pub tag: ServerMessageTag,
    pub message: ReceivedMessage,
}

#[repr(u8)]
#[derive(Debug)]
pub enum ServerMessageTag {
    ConnectRequest,
    ConnectDeny,
    Connect,
    DisconnectRequest,
    Disconnect,

    ChunkDataRequest,
    ChunkDataDeny,
    ChunkData,
    UpdateChunksRequest,
    UpdateChunksDeny,
    UpdateChunks,

    SetPositionRequest,
    SetPositionDeny,
    SetPosition,

    Ping,
    __Count,
}
impl_try_from_uint!(NetworkMessageTag => ServerMessageTag);

const MAX_CHUNKS_PER_BATCH: usize = 32;
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MsgChunkDataRequest {
    pub count: u8,
    _pad: [u8; 3],
    pub positions: [IVec3; MAX_CHUNKS_PER_BATCH],
}

impl NetworkMessageConfig for MsgChunkDataRequest {
    const TAG: NetworkMessageTag = ServerMessageTag::ChunkDataRequest as NetworkMessageTag;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MsgChunkData {
    pub position: IVec3,                                         // 0..11
    pub blocks: [VoxelBlock; CHUNK_DIM * CHUNK_DIM * CHUNK_DIM], // 12..8204
}

impl NetworkMessageConfig for MsgChunkData {
    const TAG: NetworkMessageTag = ServerMessageTag::ChunkData as NetworkMessageTag;
    const FRAGMENT_COUNT: usize = 10;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MsgSetPositionRequest {
    pub position: Vec3,
}

impl NetworkMessageConfig for MsgSetPositionRequest {
    const TAG: NetworkMessageTag = ServerMessageTag::SetPositionRequest as NetworkMessageTag;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MsgPing {
    pub byte: u8,
}

impl NetworkMessageConfig for MsgPing {
    const TAG: NetworkMessageTag = ServerMessageTag::Ping as NetworkMessageTag;
}

fn pop_network_msg_tag(data: &mut Vec<u8>) -> NetworkMessageTag {
    NetworkMessageTag::from_be_bytes(
        data.split_off(data.len() - size_of::<NetworkMessageTag>())
            .try_into()
            .unwrap(),
    )
}

pub fn process_message(mut message: ReceivedMessage) -> ServerMessage {
    let network_tag = pop_network_msg_tag(&mut message.data);
    let server_tag = ServerMessageTag::try_from(network_tag).unwrap();
    ServerMessage {
        tag: server_tag,
        message,
    }
}
