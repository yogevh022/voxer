use crate::renderer::gpu::malloc::common::{DebugFmtMemSlot, VirtualMemSlot, malloc_fmt};
use crate::renderer::gpu::malloc::virtual_malloc::{MallocError, VirtualMalloc};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;

#[derive(Clone)]
pub struct VMallocFirstFit {
    pub arena_size: usize,
    pub arena_offset: usize,
    pub free_blocks: HashMap<usize, VirtualMemSlot>,
    pub used_blocks: HashMap<usize, VirtualMemSlot>,
}

impl VMallocFirstFit {
    pub fn new(arena_size: usize, arena_offset: usize) -> Self {
        let initial_slot = VirtualMemSlot {
            size: arena_size,
            prev_free: 0,
        };
        Self {
            arena_size,
            arena_offset,
            free_blocks: HashMap::from([(arena_offset, initial_slot)]),
            used_blocks: HashMap::new(),
        }
    }
    
    pub(crate) fn alloc(
        &mut self,
        requested_allocation: usize,
    ) -> Result<usize, MallocError> {
        let available_slot = self
            .free_blocks
            .iter()
            .find_map(|(key, slot)| (slot.size >= requested_allocation).then(|| *key));
        let slot_offset = available_slot.ok_or(MallocError::OutOfMemory)?;

        let mut slot = self.free_blocks.remove(&slot_offset).unwrap();
        let leftover_size = slot.size - requested_allocation;

        if leftover_size != 0 {
            let leftover_free = VirtualMemSlot {
                size: leftover_size,
                prev_free: 0,
            };
            self.free_blocks
                .insert(slot_offset + requested_allocation, leftover_free);
        }

        self.used_blocks
            .get_mut(&(slot_offset + slot.size))
            .map(|next_slot| next_slot.prev_free = leftover_size);

        slot.size = requested_allocation;
        self.used_blocks.insert(slot_offset, slot);

        Ok(slot_offset)
    }

    pub(crate) fn free(&mut self, allocation: usize) -> Result<(), MallocError> {
        let slot_opt = self.used_blocks.remove(&allocation);
        let mut slot = slot_opt.ok_or(MallocError::InvalidAllocation)?;
        let next_index = allocation + slot.size;
        slot.size += slot.prev_free;

        let greedy_index = allocation - slot.prev_free;
        self.free_blocks.remove(&greedy_index);
        slot.prev_free = 0;
        self.free_blocks.insert(greedy_index, slot);
        self.used_blocks
            .get_mut(&next_index)
            .map(|s| s.prev_free = slot.size);
        Ok(())
    }
    
    pub(crate) fn clear(&mut self) {
        self.free_blocks.clear();
        self.used_blocks.clear();
    }

    pub fn total_free(&self) -> usize {
        self.free_blocks.iter().map(|(_, s)| s.size).sum()
    }

    pub fn free_count(&self) -> usize {
        self.free_blocks.len()
    }

    pub fn total_used(&self) -> usize {
        self.used_blocks.iter().map(|(_, s)| s.size).sum()
    }

    pub fn used_count(&self) -> usize {
        self.used_blocks.len()
    }

    pub fn debug(&self) {
        print!("\x1B[2J\x1B[1;1H{:?}", self); // the blob clears cli
        std::io::stdout().flush().unwrap();
    }
}

impl Debug for VMallocFirstFit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_dbg_slots: Vec<_> = self
            .free_blocks
            .iter()
            .map(|(o, s)| DebugFmtMemSlot {
                offset: *o,
                size: s.size,
            })
            .collect();
        sorted_dbg_slots.sort();
        malloc_fmt(
            f,
            self.arena_size - self.arena_offset,
            self.used_count(),
            self.free_count(),
            sorted_dbg_slots.iter(),
        )
    }
}
