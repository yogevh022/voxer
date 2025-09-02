use crc32fast::Hasher;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

#[derive(Debug)]
pub enum NetworkingError {
    SocketError,
    WouldBlock,
    InvalidChecksum,
}

#[derive(Debug)]
pub struct NetworkMessage {
    pub other: SocketAddr,
    pub data: Vec<u8>,
}

pub struct VoxerUdpSocket<const BUFF_SIZE: usize> {
    socket: UdpSocket,
    buffer: [u8; BUFF_SIZE],
}

impl<const BUFF_SIZE: usize> VoxerUdpSocket<BUFF_SIZE> {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Self {
        let socket = UdpSocket::bind(addr).unwrap();
        socket.set_nonblocking(true).unwrap();
        Self {
            socket,
            buffer: [0; BUFF_SIZE],
        }
    }

    pub fn bind_port(port: u16) -> Self {
        Self::bind(SocketAddr::from(([0, 0, 0, 0], port)))
    }

    pub fn bind_any() -> Self {
        Self::bind("0.0.0.0:0")
    }

    pub fn try_recv(&mut self) -> Result<NetworkMessage, NetworkingError> {
        let (n_bytes, src) = match self.socket.recv_from(&mut self.buffer) {
            Ok(result) => result,
            Err(e) if e.kind() == ErrorKind::WouldBlock => Err(NetworkingError::WouldBlock)?,
            Err(_) => Err(NetworkingError::SocketError)?,
        };

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

    pub fn send_to(&self, data: Vec<u8>, addr: &SocketAddr) -> Result<(), NetworkingError> {
        let msg_bytes = prepare_message(data);

        self.socket
            .send_to(&msg_bytes, addr)
            .ok()
            .ok_or(NetworkingError::SocketError)?;
        Ok(())
    }

    pub fn send_to_many(&self, data: Vec<u8>, addrs: &[SocketAddr]) -> Result<(), NetworkingError> {
        let msg_bytes = prepare_message(data);

        for addr in addrs {
            self.socket
                .send_to(&msg_bytes, addr)
                .ok()
                .ok_or(NetworkingError::SocketError)?;
        }
        Ok(())
    }
}

fn compute_checksum(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

fn prepare_message(mut data: Vec<u8>) -> Vec<u8> {
    let checksum = compute_checksum(&data).to_be_bytes();
    data.extend_from_slice(&checksum);
    data
}
