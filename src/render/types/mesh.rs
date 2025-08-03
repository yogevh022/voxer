use crate::render::types::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub vertex_offset: u64,
    pub index_offset: u64,
}
