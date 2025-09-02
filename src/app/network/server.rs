use std::net::{SocketAddr, UdpSocket};
use rustc_hash::FxHashMap;
use crate::app::network::networking::{NetworkMessage, Networking, NetworkingError};
use super::client::NetworkClient;

pub struct NetworkServer<const BUFF_SIZE: usize> {
    clients: FxHashMap<SocketAddr, NetworkClient<BUFF_SIZE>>,
    net: Networking<BUFF_SIZE>,
}

impl<const BUFF_SIZE: usize> NetworkServer<BUFF_SIZE> {
    pub fn new() -> Self {
        Self {
            clients: FxHashMap::default(),
            net: Networking::new(),
        }
    }
    pub fn send(&mut self, addr: &String) {
        let addr = addr.parse::<SocketAddr>().unwrap();
        let socket = UdpSocket::bind(addr).unwrap();
        // 192.168.50.215
        let other = SocketAddr::new("10.0.0.1".parse().unwrap(), addr.port());
        let data = vec![1,2,3,4,5,6,7,8,9,10];
        let msg = NetworkMessage {
            other,
            data,
        };
        
        let q = self.net.send(&socket, msg);
        dbg!(q);
    }
}