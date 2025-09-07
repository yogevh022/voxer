use glam::Vec3;

pub(crate) struct PlayerLocation {
    pub world: usize,
    pub position: Vec3,
}

pub(crate) struct PlayerSession {
    pub id: usize,
    pub name: String,
    pub location: PlayerLocation,
}