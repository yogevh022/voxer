use crate::meshing::chunk::generate_mesh;
use crate::render::types::Mesh;
use crate::texture::TextureAtlas;
use crate::world::types::Chunk;
use crossbeam::channel;
use glam::IVec3;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::sync::Arc;

pub type MeshGenRequest = Vec<(IVec3, Chunk)>;
pub type MeshGenResponse = Vec<(IVec3, Chunk)>;

pub struct MeshGenHandle {
    pub send: channel::Sender<MeshGenRequest>,
    pub receive: channel::Receiver<MeshGenResponse>,
}

pub fn world_mesh_generation_task(
    atlas: Arc<TextureAtlas>,
    send: channel::Sender<MeshGenResponse>,
    receive: channel::Receiver<MeshGenRequest>,
) {
    while let Ok(chunks) = receive.recv() {
        let generated_meshes = chunks
            .into_par_iter()
            .map(|(c_pos, mut chunk)| {
                let mesh = generate_mesh(&chunk, &atlas);
                chunk.mesh = Some(mesh);
                (c_pos, chunk)
            })
            .collect();
        send.send(generated_meshes).unwrap();
    }
}
