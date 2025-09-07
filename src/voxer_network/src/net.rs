use super::NetworkingError;
use super::fragmentation::MsgFragAssembler;
use crate::traits::NetworkSerializable;
use crate::types::{DecodedMessage, RawMsg, ReceivedMessage, SerializedMessage};
use crc32fast::Hasher;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub struct UdpChannel<const BUFF_SIZE: usize> {
    socket: UdpSocket,
    fragment_assembler: MsgFragAssembler,
    buffer: [u8; BUFF_SIZE],
}

impl<const BUFF_SIZE: usize> UdpChannel<BUFF_SIZE> {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Self {
        let socket = UdpSocket::bind(addr).unwrap();
        socket.set_nonblocking(true).unwrap();
        Self {
            socket,
            fragment_assembler: MsgFragAssembler::default(),
            buffer: [0; BUFF_SIZE],
        }
    }

    pub fn recv_single(&mut self) -> Option<ReceivedMessage> {
        let msg = if let Ok((src, raw_message)) = self.try_recv_raw() {
            match raw_message.consume() {
                DecodedMessage::Single(data) => {
                    Some(ReceivedMessage { src, data })
                }
                DecodedMessage::Fragment(fragment) => {
                    self.fragment_assembler
                        .insert_fragment(src, fragment)
                        .map(|data| ReceivedMessage { src, data })
                }
            }
        } else {
            None
        };
        self.fragment_assembler.gc_pass();
        msg
    }

    pub fn send_to<A: ToSocketAddrs>(
        &self,
        data: Box<dyn NetworkSerializable>,
        addr: &A,
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

    pub fn send_to_many<A: ToSocketAddrs>(
        &self,
        data: Box<dyn NetworkSerializable>,
        addrs: &[&A],
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

    fn send_bytes_to<A: ToSocketAddrs>(
        &self,
        data: Vec<u8>,
        addr: &A,
    ) -> Result<(), NetworkingError> {
        let msg_bytes = prepare_message(data);
        self.socket
            .send_to(&msg_bytes, addr)
            .ok()
            .ok_or(NetworkingError::SocketError)?;
        Ok(())
    }

    fn try_recv_raw(&mut self) -> Result<(SocketAddr, RawMsg<'_>), NetworkingError> {
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
