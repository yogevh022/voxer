use crate::render::types::{Mesh, Vertex};

pub struct Meshes {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}

impl Meshes {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn extend_with_offset(&mut self, mesh: &Mesh) {
        let index_offset = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&mesh.vertices);
        self.indices
            .extend(mesh.indices.iter().map(|i| *i + index_offset));
    }
}
