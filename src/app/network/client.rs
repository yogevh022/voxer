use std::net::UdpSocket;
use crate::app::network::networking::Networking;

pub struct NetworkClient<const BUFF_SIZE: usize> {
    net: Networking<BUFF_SIZE>,
}

impl<const BUFF_SIZE: usize> NetworkClient<BUFF_SIZE> {
    pub fn new() -> Self {
        Self {
            net: Networking::new()
        }
    }
    
    pub fn listen(&mut self, addr: &String) {
        let socket = UdpSocket::bind("0.0.0.0:12345").unwrap();

        self.net.listen(&socket, |msg| {
            dbg!(msg);
        });
    }
}