use crate::world::types::Chunk;
use glam::IVec3;


pub trait World {
    fn tick(&mut self);
    fn chunks_at(&self, positions: &[IVec3]) -> Vec<Chunk>;
    fn update_simulated_chunks(&mut self, origins: &[IVec3]);
    fn start_simulation(&mut self);
    fn stop_simulation(&mut self);
}
