use crate::app::network::compute_checksum;
use std::net::{SocketAddr, UdpSocket};

#[derive(Debug)]
pub enum NetworkingError {
    SocketError,
    InvalidChecksum,
}

#[derive(Debug)]
pub struct NetworkMessage {
    pub other: SocketAddr,
    pub data: Vec<u8>,
}

pub struct Networking<const BUFF_SIZE: usize> {
    buffer: [u8; BUFF_SIZE],
}

impl<const BUFF_SIZE: usize> Networking<BUFF_SIZE> {
    pub fn new() -> Self {
        Self {
            buffer: [0; BUFF_SIZE],
        }
    }
    
    fn recv(&mut self, socket: &UdpSocket) -> Result<NetworkMessage, NetworkingError> {
        let (n_bytes, src) = socket
            .recv_from(&mut self.buffer)
            .ok()
            .ok_or(NetworkingError::SocketError)?;
        let payload = &self.buffer[..n_bytes - 4];
        let received_checksum =
            u32::from_be_bytes(self.buffer[n_bytes - 4..n_bytes].try_into().unwrap());
        if compute_checksum(payload) != received_checksum {
            return Err(NetworkingError::InvalidChecksum);
        }
        Ok(NetworkMessage {
            other: src,
            data: payload.to_vec(),
        })
    }

    pub fn send(&mut self, socket: &UdpSocket, msg: NetworkMessage) -> Result<usize, NetworkingError> {
        let mut msg_bytes = msg.data;
        let checksum = compute_checksum(&msg_bytes).to_be_bytes();
        msg_bytes.extend_from_slice(&checksum);

        socket.send_to(&msg_bytes, msg.other).ok().ok_or(NetworkingError::SocketError)
    }

    pub fn listen<F>(&mut self, socket: &UdpSocket, mut on_msg: F)
    where
        F: FnMut(Result<NetworkMessage, NetworkingError>),
    {
        loop {
            let msg = self.recv(&socket);
            on_msg(msg);
        }
    }
}
