use rustc_hash::FxHashMap;
use crate::renderer::gpu::VirtualMalloc;
use crate::renderer::gpu::malloc::VirtualMemSlot;
use crate::renderer::gpu::malloc::virtual_malloc::{
    MallocError, SimpleAllocation, SimpleAllocationRequest,
};

#[derive(Clone)]
pub struct VMallocFirstFit {
    pub arena_size: usize,
    pub arena_offset: usize,
    free_blocks: FxHashMap<usize, VirtualMemSlot>,
    used_blocks: FxHashMap<usize, VirtualMemSlot>,
}

impl VirtualMalloc for VMallocFirstFit {
    type AllocationRequest = SimpleAllocationRequest;
    type Allocation = SimpleAllocation;
    fn new(arena_size: usize, arena_offset: usize) -> Self {
        let initial_slot = VirtualMemSlot {
            size: arena_size,
            prev_free: 0,
        };
        let mut free_blocks = FxHashMap::default();
        free_blocks.insert(arena_offset, initial_slot);
        Self {
            arena_size,
            arena_offset,
            free_blocks,
            used_blocks: FxHashMap::default(),
        }
    }

    fn alloc(
        &mut self,
        allocation_request: Self::AllocationRequest,
    ) -> Result<Self::Allocation, MallocError> {
        let available_slot = self
            .free_blocks
            .iter()
            .find_map(|(key, slot)| (slot.size >= allocation_request.size).then(|| *key));
        let slot_offset = available_slot.ok_or(MallocError::OutOfMemory)?;

        let mut slot = self.free_blocks.remove(&slot_offset).unwrap();
        let leftover_size = slot.size - allocation_request.size;

        if leftover_size != 0 {
            let leftover_free = VirtualMemSlot {
                size: leftover_size,
                prev_free: 0,
            };
            self.free_blocks
                .insert(slot_offset + allocation_request.size, leftover_free);
        }

        self.used_blocks
            .get_mut(&(slot_offset + slot.size))
            .map(|next_slot| next_slot.prev_free = leftover_size);

        slot.size = allocation_request.size;
        self.used_blocks.insert(slot_offset, slot);

        Ok(Self::Allocation {
            offset: slot_offset,
        })
    }

    fn free(&mut self, allocation: Self::Allocation) -> Result<(), MallocError> {
        let slot_opt = self.used_blocks.remove(&allocation.offset);
        let mut slot = slot_opt.ok_or(MallocError::InvalidAllocation)?;
        let next_index = allocation.offset + slot.size;
        slot.size += slot.prev_free;

        let greedy_index = allocation.offset - slot.prev_free;
        self.free_blocks.remove(&greedy_index);
        slot.prev_free = 0;
        // todo clean this mess
        self.used_blocks
            .get_mut(&next_index)
            .map(|s| s.prev_free = slot.size)
            .unwrap_or_else(|| {
                self.free_blocks
                    .remove(&next_index)
                    .map(|s| {
                        slot.size += s.size;
                        self.used_blocks.get_mut(&(next_index + s.size)).map(|us| {
                            us.prev_free = slot.size;
                        });
                    });
            });
        self.free_blocks.insert(greedy_index, slot);
        Ok(())
    }

    fn clear(&mut self) {
        self.free_blocks.clear();
        self.used_blocks.clear();
    }

    fn total_size(&self) -> usize {
        self.arena_size
    }

    fn available_size(&self) -> usize {
        self.free_blocks.iter().map(|(_, s)| s.size).sum()
    }

    fn available_count(&self) -> usize {
        self.free_blocks.len()
    }

    fn used_size(&self) -> usize {
        self.used_blocks.iter().map(|(_, s)| s.size).sum()
    }

    fn used_count(&self) -> usize {
        self.used_blocks.len()
    }
}
