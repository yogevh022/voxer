use crate::world::types::{Chunk, ChunkBlocks};
use glam::IVec3;
use slab::Slab;
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct ChunkManager {
    pending_load: Vec<IVec3>,
    pending_unload: Vec<usize>,
    pending_generation: HashSet<IVec3>,
    pub chunks: HashMap<IVec3, Option<Chunk>>,
    pub loaded_chunks: Slab<IVec3>,
}

impl ChunkManager {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pending_load: Vec::new(),
            pending_unload: Vec::new(),
            pending_generation: HashSet::new(),
            chunks: HashMap::new(),
            loaded_chunks: Slab::with_capacity(capacity),
        }
    }

    pub fn insert_chunks(&mut self, chunks: Vec<(IVec3, Chunk)>) {
        chunks.into_iter().for_each(|(c_pos, c)| {
            self.chunks.insert(c_pos, Some(c));
            self.pending_generation.remove(&c_pos);
        });
    }

    pub fn enqueue_pending_load(&mut self, to_load: Vec<IVec3>) {
        self.pending_load.extend(to_load);
    }

    pub fn enqueue_pending_unload(&mut self, to_unload: Vec<usize>) {
        self.pending_unload.extend(to_unload);
    }

    pub fn enqueue_pending_generation(&mut self, to_generate: &Vec<IVec3>) {
        self.pending_generation.extend(to_generate);
    }

    pub fn dequeue_pending_load(&mut self) -> Vec<&ChunkBlocks> {
        let mut to_load = Vec::new();
        std::mem::swap(&mut to_load, &mut self.pending_load);
        let mut chunk_data = Vec::new();
        for tl in to_load.into_iter() {
            let chunk = self.chunks.get(&tl).as_ref().unwrap().as_ref().unwrap();
            chunk_data.push(&chunk.blocks);
            self.loaded_chunks.insert(tl);
        }

        chunk_data
    }
    pub fn dequeue_pending_unload(&mut self) -> Vec<usize> {
        let mut to_unload = Vec::new();
        std::mem::swap(&mut to_unload, &mut self.pending_unload);
        to_unload.iter().for_each(|c_idx| {
            self.loaded_chunks.remove(*c_idx);
        });
        to_unload
    }

    pub fn ungenerated_chunks_at_positions(&self, chunk_positions: &HashSet<IVec3>) -> Vec<IVec3> {
        chunk_positions
            .into_iter()
            .filter(|c_pos| {
                !self.chunks.contains_key(*c_pos) && !self.pending_generation.contains(c_pos)
            })
            .map(|c_pos| *c_pos)
            .collect()
    }

    pub fn unloaded_chunks_at_positions(&self, chunk_positions: &HashSet<IVec3>) -> Vec<IVec3> {
        // filter chunks that are not loaded from the given positions
        chunk_positions
            .into_iter()
            .filter(|c_pos| {
                !self.loaded_chunks.iter().any(|(_, &cp)| cp == **c_pos)
                    && !self.pending_load.contains(*c_pos)
                    && !self.pending_generation.contains(*c_pos)
            })
            .map(|c_pos| *c_pos)
            .collect()
    }

    pub fn loaded_chunks_at_positions(&self, chunk_positions: &HashSet<IVec3>) -> Vec<usize> {
        // filter chunks that are loaded from the given positions
        let mut unloaded_chunks: Vec<usize> = self
            .loaded_chunks
            .iter()
            .filter(|(i, c_pos)| {
                !chunk_positions.contains(*c_pos) && !self.pending_unload.contains(i)
            })
            .map(|(c_idx, _)| c_idx)
            .collect();
        unloaded_chunks.sort();
        unloaded_chunks
    }

    // pub fn unload_chunk(&mut self, chunk_position: IVec3) {
    //     self.loaded_chunks.swap_remove(&chunk_position);
    // }
    //
    // pub fn load_chunk(&mut self, chunk_position: IVec3) {
    //     self.loaded_chunks.insert(chunk_position);
    // }
}
