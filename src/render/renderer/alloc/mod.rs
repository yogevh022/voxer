use crate::render::types::{Index, Mesh, Vertex};
use crate::types::SceneObject;

pub fn size_of_mesh(mesh: Mesh) -> (u64, u64) {
    // returns (vertex_alloc, index_alloc) in bytes
    (
        (mesh.vertices.len() * size_of::<Vertex>()) as u64,
        (mesh.indices.len() * size_of::<Index>()) as u64,
    )
}

pub fn size_of_meshes(meshes: &[Mesh]) -> (u64, u64) {
    // returns (vertex_alloc, index_alloc) in bytes for all meshes in the array
    let (vert_count, ind_count) = meshes.iter().fold((0u64, 0u64), |acc, mesh| {
        (
            acc.0 + mesh.vertices.len() as u64,
            acc.1 + mesh.indices.len() as u64,
        )
    });
    (
        vert_count * size_of::<Vertex>() as u64,
        ind_count * size_of::<Index>() as u64,
    )
}

pub fn size_of_meshes_from_sos(scene_objects: &[SceneObject]) -> (u64, u64) {
    // returns (vertex_alloc, index_alloc) in bytes for all SceneObject meshes in the array
    let (vert_count, ind_count) = scene_objects.iter().fold((0u64, 0u64), |acc, so| {
        (
            acc.0 + so.model.mesh.vertices.len() as u64,
            acc.1 + so.model.mesh.indices.len() as u64,
        )
    });
    (
        vert_count * size_of::<Vertex>() as u64,
        ind_count * size_of::<Index>() as u64,
    )
}
