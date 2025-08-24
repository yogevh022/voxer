mod debug_fmt;
pub use debug_fmt::{DebugFmtMemSlot, malloc_fmt};

use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy)]
pub(crate) struct VirtualMemSlotKey {
    pub offset: usize,
    pub size: usize,
}

impl Eq for VirtualMemSlotKey {}

impl PartialEq for VirtualMemSlotKey {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
    }
}

impl VirtualMemSlotKey {
    pub fn query(offset: usize) -> Self {
        // fake size, unused for querying
        Self { offset, size: 0 }
    }

    pub fn store(offset: usize, size: usize) -> Self {
        Self { offset, size }
    }
}

impl Ord for VirtualMemSlotKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size.cmp(&other.size)
    }
}

impl PartialOrd<Self> for VirtualMemSlotKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for VirtualMemSlotKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VirtualMemSlot {
    pub size: usize,
    pub prev_free: usize,    // available adjacent space previous to this slot
}
