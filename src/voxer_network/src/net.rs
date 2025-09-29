use super::NetworkingError;
use super::fragmentation::MsgFragAssembler;
use crate::traits::NetworkSerializable;
use crate::types::{DecodedMessage, RawMsg, ReceivedMessage, SerializedMessage};
use crc32fast::Hasher;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub struct UdpChannel {
    socket: UdpSocket,
    fragment_assembler: MsgFragAssembler,
    buffer: Box<[u8]>,
}

impl UdpChannel {
    pub fn bind(addr: impl ToSocketAddrs, buffer_size: usize) -> Self {
        let socket = UdpSocket::bind(addr).unwrap();
        Self {
            socket,
            fragment_assembler: MsgFragAssembler::default(),
            buffer: vec![0; buffer_size].into_boxed_slice(),
        }
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            socket: self.socket.try_clone().unwrap(),
            fragment_assembler: MsgFragAssembler::default(),
            buffer: vec![0; self.buffer.len()].into_boxed_slice(),
        }
    }

    pub fn recv_single(&mut self) -> Option<ReceivedMessage> {
        let msg = if let Ok((src, raw_message)) = self.recv_raw() {
            match raw_message.consume() {
                DecodedMessage::Single(data) => Some(ReceivedMessage { src, data }),
                DecodedMessage::Fragment(fragment) => self
                    .fragment_assembler
                    .insert_fragment(src, fragment)
                    .map(|data| ReceivedMessage { src, data }),
            }
        } else {
            None
        };
        self.fragment_assembler.gc_pass();
        msg
    }

    pub fn send_to(
        &self,
        data: Box<dyn NetworkSerializable>,
        addr: &impl ToSocketAddrs,
    ) -> Result<(), NetworkingError> {
        match data.serialize() {
            SerializedMessage::Single(single) => {
                self.send_bytes_to(single, addr)?;
            }
            SerializedMessage::Fragmented(fragments) => {
                for fragment in fragments {
                    self.send_bytes_to(fragment, addr)?;
                }
            }
        }
        Ok(())
    }

    pub fn send_to_many(
        &self,
        data: Box<dyn NetworkSerializable>,
        addrs: &[&impl ToSocketAddrs],
    ) -> Result<(), NetworkingError> {
        match data.serialize() {
            SerializedMessage::Single(single) => {
                for addr in addrs {
                    self.send_bytes_to(single.clone(), addr)?;
                }
            }
            SerializedMessage::Fragmented(fragments) => {
                for addr in addrs {
                    for fragment in fragments.iter() {
                        self.send_bytes_to(fragment.clone(), addr)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn send_bytes_to(
        &self,
        data: Vec<u8>,
        addr: &impl ToSocketAddrs,
    ) -> Result<(), NetworkingError> {
        let msg_bytes = prepare_message(data);
        self.socket
            .send_to(&msg_bytes, addr)
            .ok()
            .ok_or(NetworkingError::SocketError)?;
        Ok(())
    }

    fn recv_raw(&mut self) -> Result<(SocketAddr, RawMsg<'_>), NetworkingError> {
        let (n_bytes, src) = match self.socket.recv_from(&mut self.buffer) {
            Ok(result) => result,
            Err(_) => Err(NetworkingError::SocketError)?,
        };

        let payload = &self.buffer[..n_bytes - 4];
        let received_checksum =
            u32::from_be_bytes(self.buffer[n_bytes - 4..n_bytes].try_into().unwrap());
        if compute_checksum(payload) != received_checksum {
            return Err(NetworkingError::InvalidChecksum);
        }
        Ok((src, RawMsg { data: payload }))
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
