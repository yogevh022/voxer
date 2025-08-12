use std::collections::HashMap;

pub struct MemoryAllocator {
    pub size: usize,
    pub free_blocks: HashMap<usize, usize>,
}

impl MemoryAllocator {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            free_blocks: HashMap::from([(0, size)]),
        }
    }

    pub fn alloc_ff(&mut self, size: usize) -> Result<usize, &str> {
        let ff_slot = self
            .free_blocks
            .iter()
            .find(|&(_, block_size)| *block_size >= size);
        let (&offset, &slot_size) = ff_slot.ok_or("No free space")?;
        self.free_blocks.insert(offset, slot_size - size);
        Ok(offset)
    }

    pub fn free(&mut self, offset: usize, size: usize) {
        let next_slot = self.free_blocks.remove(&(offset + size));
        if let Some(next_slot_size) = next_slot {
            self.free_blocks.insert(offset, size + next_slot_size);
        } else {
            self.free_blocks.insert(offset, size);
        }
    }
}
