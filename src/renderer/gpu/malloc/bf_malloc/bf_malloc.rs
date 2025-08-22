use crate::renderer::gpu::malloc::bf_malloc::types::{MemSlot, MemSlotKey};
use crate::renderer::gpu::malloc::common::{DebugFmtMemSlot, VirtualMemSlot, malloc_fmt};
use crate::renderer::gpu::malloc::virtual_malloc::{MallocError, VirtualMalloc, VirtualMallocType};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::io::Write;

pub struct VMallocBestFit {
    pub arena_size: usize,
    pub arena_offset: usize,
    pub free_blocks: BTreeMap<MemSlotKey, MemSlot>,
    pub free_blocks_by_offset: BTreeMap<usize, MemSlot>,
    pub used_blocks: HashMap<MemSlotKey, MemSlot>,
    id_counter: usize,
}

impl VMallocBestFit {
    fn new(arena_size: usize, arena_offset: usize) -> Self {
        let initial_slot_key = MemSlotKey {
            size: arena_size - arena_offset,
            id: 0,
        };
        let initial_slot = MemSlot {
            offset: arena_offset,
            owner_index: None,
        };
        Self {
            arena_size,
            arena_offset,
            free_blocks: BTreeMap::from([(initial_slot_key, initial_slot.clone())]),
            free_blocks_by_offset: BTreeMap::from([(arena_offset, initial_slot)]),
            used_blocks: HashMap::new(),
            id_counter: 1,
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }

    fn alloc(&mut self, size: usize) -> Result<MemSlot, MallocError> {
        let available_slot_opt = self.free_blocks.keys().find(|k| k.size >= size);
        let &slot_key = available_slot_opt.ok_or(MallocError::OutOfMemory)?;
        let slot = self.free_blocks.remove(&slot_key).unwrap();
        let _ = self.free_blocks_by_offset.remove(&slot.offset);
        let leftover_key = MemSlotKey {
            size: slot_key.size - size,
            id: self.next_id(),
        };
        let leftover_slot = MemSlot {
            offset: slot.offset + size,
            owner_index: None,
        };
        if leftover_key.size > 0 {
            self.free_blocks.insert(leftover_key, leftover_slot);
            self.free_blocks_by_offset.insert(leftover_slot.offset, leftover_slot);
        }
        let slot_key = MemSlotKey {
            size,
            id: self.next_id(),
        };
        self.used_blocks.insert(slot_key, slot);
        Ok(slot)
    }

    fn free(&mut self, slot_key: MemSlotKey, owner: Option<usize>) -> Result<(), MallocError> {
        // merge blocks
        let mut slot = self.used_blocks.remove(&slot_key).unwrap();
        slot.owner_index = owner;
        self.free_blocks.insert(slot_key, slot);
        self.free_blocks_by_offset.insert(slot.offset, slot);
        Ok(())
    }

    fn total_free(&self) -> usize {
        self.free_blocks.iter().map(|(k, _)| k.size).sum()
    }

    fn free_count(&self) -> usize {
        self.free_blocks.len()
    }

    fn total_used(&self) -> usize {
        self.used_blocks.iter().map(|(k, _)| k.size).sum()
    }

    fn used_count(&self) -> usize {
        self.used_blocks.len()
    }

    fn debug(&self) {
        print!("\x1B[2J\x1B[1;1H{:?}", self); // the blob clears cli
        std::io::stdout().flush().unwrap();
    }
}

impl Debug for VMallocBestFit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_dbg_slots: Vec<_> = self
            .free_blocks
            .iter()
            .map(|(k, s)| DebugFmtMemSlot {
                offset: s.offset,
                size: k.size,
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
