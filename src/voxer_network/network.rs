use crate::voxer_network::fragment::FragmentAssembler;
use crate::voxer_network::message::{NetworkDeserializable, NetworkMessage, NetworkRawMessage, NetworkReceiveMessage, NetworkSendMessage, NetworkSerializable};
use crc32fast::Hasher;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

#[derive(Debug)]
pub enum NetworkingError {
    SocketError,
    WouldBlock,
    InvalidChecksum,
}

pub struct VoxerUdpSocket<const BUFF_SIZE: usize> {
    socket: UdpSocket,
    fragment_assembler: FragmentAssembler,
    buffer: [u8; BUFF_SIZE],
}

impl<const BUFF_SIZE: usize> VoxerUdpSocket<BUFF_SIZE> {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Self {
        let socket = UdpSocket::bind(addr).unwrap();
        socket.set_nonblocking(true).unwrap();
        Self {
            socket,
            fragment_assembler: FragmentAssembler::default(),
            buffer: [0; BUFF_SIZE],
        }
    }

    pub fn bind_port(port: u16) -> Self {
        Self::bind(SocketAddr::from(([0, 0, 0, 0], port)))
    }

    pub fn bind_any() -> Self {
        Self::bind("0.0.0.0:0")
    }

    fn try_recv(&mut self) -> Result<(SocketAddr, NetworkRawMessage), NetworkingError> {
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
        Ok((src, NetworkRawMessage::new(payload)))
    }

    pub fn full_recv(&mut self) -> Vec<NetworkMessage> {
        let mut received_messages = Vec::new();
        while let Ok((src, raw_message)) = self.try_recv() {
            match raw_message.deserialize() {
                NetworkReceiveMessage::Single(single) => {
                    received_messages.push(NetworkMessage::new(src, single))
                }
                NetworkReceiveMessage::Fragment(fragment) => {
                    self.fragment_assembler
                        .insert_fragment(fragment)
                        .map(|msg| received_messages.push(NetworkMessage::new(src, msg)));
                }
            };
        }
        received_messages
    }

    pub fn send_to<S: NetworkSerializable, A: ToSocketAddrs>(&self, data: S, addr: &A) -> Result<(), NetworkingError> {
        match data.serialize() {
            NetworkSendMessage::Single(single) => {
                self.send_bytes_to(single, addr)?;
            }
            NetworkSendMessage::Fragmented(fragments) => {
                for fragment in fragments {
                    self.send_bytes_to(fragment, addr)?;
                }
            }
        }
        Ok(())
    }

    pub fn send_to_many(&self, data: Vec<u8>, addrs: &[SocketAddr]) -> Result<(), NetworkingError> {
        todo!();
        let msg_bytes = prepare_message(data);

        for addr in addrs {
            self.socket
                .send_to(&msg_bytes, addr)
                .ok()
                .ok_or(NetworkingError::SocketError)?;
        }
        Ok(())
    }

    fn send_bytes_to<A: ToSocketAddrs>(&self, data: Vec<u8>, addr: &A) -> Result<(), NetworkingError> {
        let msg_bytes = prepare_message(data);
        self.socket
            .send_to(&msg_bytes, addr)
            .ok()
            .ok_or(NetworkingError::SocketError)?;
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
