use crate::render::types::{Index, Vertex};

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
    pub vertex_offset: u64,
    pub index_offset: u64,
}
