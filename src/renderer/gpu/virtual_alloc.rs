use crate::compute;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::io::Write;

pub struct VirtualMemAlloc {
    pub size: usize,
    free_blocks: HashMap<usize, VirtualMemSlot>,
    allocated_blocks: HashMap<usize, VirtualMemSlot>,
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualMemSlot {
    pub size: usize,
    pub prev_free: Option<usize>, // immediate index of previous free slot
}

impl VirtualMemAlloc {
    pub fn new(size: usize) -> Self {
        let initial_slot = VirtualMemSlot {
            size: size,
            prev_free: None,
        };
        Self {
            size,
            free_blocks: HashMap::from([(0, initial_slot)]),
            allocated_blocks: HashMap::new(),
        }
    }

    pub fn alloc(&mut self, size: usize) -> Result<usize, &str> {
        let available_slot = self
            .free_blocks
            .iter()
            .find(|(_, slot)| slot.size >= size)
            .map(|(i, _)| *i);
        let offset = self.alloc_slot(available_slot, size)?;
        Ok(offset)
    }

    pub fn free(&mut self, offset: usize) {
        let mut slot = self.allocated_blocks.remove(&offset).unwrap();
        let next_slot = self.free_blocks.remove(&(offset + slot.size));
        slot.size += next_slot.map(|s| s.size).unwrap_or(0);
        let prev_free = slot.prev_free.take().unwrap_or(offset);
        self.free_blocks.insert(prev_free, slot);
    }

    pub fn alloc_slot(
        &mut self,
        available_slot: Option<usize>,
        size: usize,
    ) -> Result<usize, &str> {
        let offset = available_slot.ok_or("No free space")?;
        let mut slot = self.free_blocks.remove(&offset).unwrap();

        let leftover = slot.size - size;
        slot.size = size;
        self.allocated_blocks.insert(offset, slot);

        let next_slot_prev_free = (leftover != 0).then(|| {
            let next_free = VirtualMemSlot {
                size: leftover,
                prev_free: None,
            };
            self.free_blocks.insert(offset + size, next_free);
            offset + size
        });

        self.allocated_blocks
            .get_mut(&(offset + size))
            .map(|next_free| next_free.prev_free = next_slot_prev_free);

        Ok(offset)
    }

    pub fn draw_cli(&self) {
        print!("\x1B[2J\x1B[1;1H{:?}", self); // the blob clears cli
        std::io::stdout().flush().unwrap();
    }
}

fn push_bar_char(s: &mut String, char: char, size: &mut usize, row_size: usize) {
    if *size % row_size == 0 {
        s.push('\n');
    }
    s.push(char);
    *size += 1;
}

impl Debug for VirtualMemAlloc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let resolution = self.size / (compute::KIB >> 1);
        const ROWS: usize = 10;
        let row_size = (self.size / resolution) / ROWS;
        let mut mem_bar = String::with_capacity((self.size / resolution) + ROWS);
        let mut current_bar_size = 0;
        for (&offset, slot) in self.free_blocks.iter().collect::<BTreeMap<_, _>>() {
            if offset / resolution != current_bar_size {
                for _ in 0..(offset / resolution) - current_bar_size {
                    push_bar_char(&mut mem_bar, '#', &mut current_bar_size, row_size);
                }
            }
            for _ in 0..slot.size / resolution {
                push_bar_char(&mut mem_bar, '_', &mut current_bar_size, row_size);
            }
        }
        mem_bar.push_str(
            "*".repeat((self.size / resolution) - current_bar_size)
                .as_str(),
        );
        let used_bytes = self
            .allocated_blocks
            .iter()
            .map(|(_, slot)| slot.size)
            .sum::<usize>();
        write!(
            f,
            "~used: {:>6} / {:>6}\n\
            alloc: {:>6} ~avg: {:>6}\n\
            free:  {:>6} ~avg: {:>6}\n\
            {}",
            compute::bytes::repr_bytes(used_bytes),
            compute::bytes::repr_bytes(self.size),
            self.allocated_blocks.len(),
            compute::bytes::repr_bytes(used_bytes / self.allocated_blocks.len().max(1)),
            self.free_blocks.len(),
            compute::bytes::repr_bytes((self.size - used_bytes) / self.free_blocks.len().max(1)),
            mem_bar
        )
    }
}
