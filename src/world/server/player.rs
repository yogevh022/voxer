use std::net::IpAddr;
use glam::Vec3;

pub struct PlayerLocation {
    pub world: usize,
    pub position: Vec3,
}

pub struct PlayerNetwork {
    pub addr: IpAddr,
}

pub struct Player {
    pub id: usize,
    pub name: String,
    pub location: PlayerLocation,
    pub network: PlayerNetwork,
}