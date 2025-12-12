use crate::compute::throttler::{PositionThrottler, Throttler};
use crate::world::network::{
    MsgChunkDataRequest, MsgConnectRequest, MsgSetPositionRequest, NetworkHandle, ServerMessage,
};
use glam::{IVec3, Vec3};
use std::net::SocketAddr;
use std::time::Instant;

pub struct ClientWorldNetwork {
    network_handle: NetworkHandle,
    chunk_request_throttler: PositionThrottler,
    chunk_request_batch: Vec<IVec3>,
    message_buffer: Vec<ServerMessage>,
    server_addr: Option<SocketAddr>,
}

impl ClientWorldNetwork {
    pub(crate) fn new(mut network_handle: NetworkHandle) -> Self {
        network_handle.listen();
        Self {
            network_handle,
            chunk_request_throttler: PositionThrottler::new(
                (1 << 18) + 1,
                std::time::Duration::from_millis(200),
            ),
            chunk_request_batch: Vec::new(), // fixme capacity
            message_buffer: Vec::new(),      // fixme capacity
            server_addr: None,
        }
    }

    pub(crate) fn receive_messages<F: FnMut(ServerMessage)>(&mut self, mut f: F) {
        let message_buffer = &mut self.message_buffer;
        message_buffer.clear();
        self.network_handle.take_messages_out(64, message_buffer);
        for msg in message_buffer.drain(..) {
            f(msg);
        }
    }

    fn server_addr(&self) -> &SocketAddr {
        self.server_addr.as_ref().unwrap()
    }

    pub(crate) fn prepare_to_batch_requests(&mut self) {
        self.chunk_request_batch.clear();
        self.chunk_request_throttler.set_now(Instant::now());
    }

    pub(crate) fn batch_chunk_request(&mut self, chunk_position: IVec3) {
        if self.chunk_request_throttler.request(chunk_position) {
            self.chunk_request_batch.push(chunk_position);
        }
    }

    pub(crate) fn request_chunk_batch(&mut self) {
        if self.chunk_request_batch.is_empty() {
            return;
        }
        let chunk_data_request = MsgChunkDataRequest::with_positions(&self.chunk_request_batch);
        let msg = Box::new(chunk_data_request);
        self.network_handle
            .send_to(msg, self.server_addr())
            .unwrap();
    }

    pub fn send_player_position(&self, position: Vec3) {
        let set_position_request = MsgSetPositionRequest { position };
        let msg = Box::new(set_position_request);
        self.network_handle
            .send_to(msg, self.server_addr())
            .unwrap();
    }

    pub fn send_connection_request(&self, server_addr: SocketAddr) {
        let connection_request = MsgConnectRequest { byte: 62 };
        let msg = Box::new(connection_request);
        self.network_handle.send_to(msg, &server_addr).unwrap();
    }

    pub fn set_server_addr(&mut self, server_addr: SocketAddr) {
        self.server_addr = Some(server_addr);
    }
}
