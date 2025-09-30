mod handle;

use crate::impl_try_from_uint;
use crate::voxer_network::{NetworkMessageTag, ReceivedMessage};
use voxer_macros::network_message;
use crate::world::types::{ChunkBlocks};
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, Vec3};
pub use handle::NetworkHandle;

#[derive(Debug)]
pub struct ServerMessage {
    pub tag: ServerMessageTag,
    pub message: ReceivedMessage,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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

impl ServerMessageTag {
    pub fn as_tag(&self) -> NetworkMessageTag {
        *self as NetworkMessageTag
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[network_message(tag = ServerMessageTag::ConnectRequest.as_tag())]
pub struct MsgConnectRequest {
    pub byte: u8,
}

// todo find a better place for consts like this
pub(crate) const MAX_CHUNKS_PER_BATCH: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[network_message(tag = ServerMessageTag::ChunkDataRequest.as_tag())]
pub struct MsgChunkDataRequest {
    pub count: u8,
    _pad: [u8; 3],
    pub positions: [IVec3; MAX_CHUNKS_PER_BATCH],
}

impl MsgChunkDataRequest {
    pub fn new() -> Self {
        Self {
            count: 0,
            _pad: [0; 3],
            positions: [IVec3::default(); MAX_CHUNKS_PER_BATCH],
        }
    }

    pub fn new_with_positions(positions: &[IVec3]) -> Self {
        let mut request = Self::new();
        request.count = positions.len() as u8;
        request.positions[..positions.len()].copy_from_slice(positions);
        request
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[network_message(tag = ServerMessageTag::ChunkData.as_tag() frags = 10)]
pub struct MsgChunkData {
    pub position: IVec3,     // 0..11
    pub solid_count: u32,    // 12..15
    pub blocks: ChunkBlocks, // 16..8208
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[network_message(tag = ServerMessageTag::SetPositionRequest.as_tag())]
pub struct MsgSetPositionRequest {
    pub position: Vec3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[network_message(tag = ServerMessageTag::Ping.as_tag())]
pub struct MsgPing {
    pub byte: u8,
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
