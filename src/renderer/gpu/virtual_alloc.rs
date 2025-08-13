use crate::compute;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

pub struct VirtualMemAlloc {
    pub size: usize,
    free_blocks: HashMap<usize, usize>,
    allocated_blocks: HashMap<usize, usize>,
}

impl VirtualMemAlloc {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            free_blocks: HashMap::from([(0, size)]),
            allocated_blocks: HashMap::new(),
        }
    }

    pub fn with_offset(size: usize, offset: usize) -> Self {
        Self {
            size,
            free_blocks: HashMap::from([(offset, size)]),
            allocated_blocks: HashMap::new(),
        }
    }

    pub fn alloc(&mut self, size: usize) -> Result<usize, &str> {
        let ff_slot = self
            .free_blocks
            .iter()
            .find(|&(_, block_size)| *block_size >= size);
        let (&offset, &slot_size) = ff_slot.ok_or("No free space")?;
        self.free_blocks.remove(&offset);
        self.free_blocks.insert(offset + size, slot_size - size);
        self.allocated_blocks.insert(offset, size);
        Ok(offset)
    }

    pub fn free(&mut self, offset: usize) {
        let size = self.allocated_blocks.remove(&offset).unwrap();
        let next_slot = self.free_blocks.remove(&(offset + size));
        if let Some(next_slot_size) = next_slot {
            self.free_blocks.insert(offset, size + next_slot_size);
        } else {
            self.free_blocks.insert(offset, size);
        }
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
        const KIB_16: usize = compute::KIB * 16;
        const ROWS: usize = 16;
        let row_size = (self.size / KIB_16) / ROWS;
        let mut mem_bar = String::with_capacity((self.size / KIB_16) + ROWS);
        let mut current_bar_size = 0;
        for (&offset, &size) in self.free_blocks.iter().collect::<BTreeMap<_, _>>() {
            if offset / KIB_16 != current_bar_size {
                for _ in 0..(offset / KIB_16) - current_bar_size {
                    push_bar_char(&mut mem_bar, '#', &mut current_bar_size, row_size);
                }
            }
            for _ in 0..size / KIB_16 {
                push_bar_char(&mut mem_bar, '_', &mut current_bar_size, row_size);
            }
        }
        mem_bar.push_str("&".repeat((self.size / KIB_16) - current_bar_size).as_str());
        let used_kb = self
            .allocated_blocks
            .iter()
            .map(|(_, size)| *size)
            .sum::<usize>()
            / compute::KIB;
        write!(
            f,
            "~used: {}kb, alloc: {}, free count: {}\n{}",
            used_kb,
            self.allocated_blocks.len(),
            self.free_blocks.len(),
            mem_bar
        )
    }
}
