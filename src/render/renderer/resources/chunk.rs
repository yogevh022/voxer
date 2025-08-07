use crate::render::renderer::resources::MeshBuffers;
use glam::{IVec3, Mat4};
use indexmap::IndexMap;
use std::ops::Index;

pub struct ChunkPoolEntry {
    pub mesh_buffers: MeshBuffers,
    // pub position: IVec3,
    pub index_offset: u32,
}

#[derive(Default)]
pub struct ChunkPoolOperations {
    remove: Vec<usize>,
    load: IndexMap<IVec3, ChunkPoolEntry>,
}

#[derive(Default)]
pub struct ChunkPool {
    chunks: IndexMap<IVec3, ChunkPoolEntry>,
    pending_operations: ChunkPoolOperations,
}

impl ChunkPool {
    #[inline(always)]
    pub fn queue_load(&mut self, chunk_pos: IVec3, chunk_entry: ChunkPoolEntry) {
        self.pending_operations.load.insert(chunk_pos, chunk_entry);
    }
    #[inline(always)]
    pub fn queue_remove(&mut self, index: usize) {
        self.pending_operations.remove.push(index);
    }
    #[inline(always)]
    pub fn take_load_queue(&mut self) -> IndexMap<IVec3, ChunkPoolEntry> {
        std::mem::take(&mut self.pending_operations.load)
    }
    #[inline(always)]
    pub fn take_remove_queue(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.pending_operations.remove)
    }

    pub fn post_swap_remove_indices(size: usize, indices: &[usize]) -> Vec<(usize, usize)> {
        // returns (index of new index -> old index pos) after simulating swap_remove
        let mut index_vec = (0..size).collect::<Vec<usize>>();
        for i in indices {
            index_vec.swap_remove(*i);
        }

        index_vec
            .into_iter()
            .enumerate()
            .filter(|(i, v)| i != v)
            .collect()
    }

    pub fn get(&self, chunk_pos: &IVec3) -> Option<&ChunkPoolEntry> {
        self.chunks.get(chunk_pos)
    }

    pub fn get_index(&self, index: usize) -> (&IVec3, &ChunkPoolEntry) {
        self.chunks.get_index(index).unwrap()
    }

    pub fn contains(&self, chunk_pos: &IVec3) -> bool {
        self.chunks.contains_key(chunk_pos)
    }

    pub fn extend(&mut self, chunks: IndexMap<IVec3, ChunkPoolEntry>) {
        self.chunks.extend(chunks);
    }

    pub fn swap_remove(&mut self, key: usize) {
        self.chunks.swap_remove_index(key);
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, IVec3, ChunkPoolEntry> {
        self.chunks.iter()
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }
}
