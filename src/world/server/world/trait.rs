use crate::world::types::Chunk;
use glam::IVec3;


pub trait World {
    fn tick(&mut self);
    fn request_chunks(&mut self, positions: &[IVec3]) -> Vec<&Chunk>;
    fn request_chunk_generation(&mut self);
    fn start_simulation(&mut self);
    fn stop_simulation(&mut self);
}
