use crate::render::types::{Index, Mesh, Vertex};
use crate::types::SceneObject;

pub fn size_of_mesh(mesh: &Mesh) -> (u64, u64) {
    // returns (vertex_alloc, index_alloc) in bytes
    (
        (mesh.vertices.len() * size_of::<Vertex>()) as u64,
        (mesh.indices.len() * size_of::<Index>()) as u64,
    )
}
