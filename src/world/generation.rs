use crate::world::types::{Chunk, World};
use crossbeam::channel;
use glam::IVec3;
use noise::OpenSimplex;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

pub type WorldGenRequest = Vec<IVec3>;
pub type WorldGenResponse = Vec<(IVec3, Chunk)>;

pub struct WorldGenHandle {
    pub send: channel::Sender<WorldGenRequest>,
    pub receive: channel::Receiver<WorldGenResponse>,
}

pub fn world_generation_task(
    seed: u32,
    send: channel::Sender<WorldGenResponse>,
    receive: channel::Receiver<WorldGenRequest>,
) {
    while let Ok(chunk_positions) = receive.recv() {
        let generated_chunks = chunk_positions
            .into_par_iter()
            .map(|chunk_pos| {
                (
                    chunk_pos,
                    World::generate_chunk(OpenSimplex::new(seed), chunk_pos),
                )
            })
            .collect();
        send.send(generated_chunks).unwrap();
    }
}
