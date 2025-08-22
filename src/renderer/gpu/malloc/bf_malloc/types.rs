use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MemSlotKey {
    pub size: usize,
    pub id: usize,
}

impl Ord for MemSlotKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size
            .cmp(&other.size)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for MemSlotKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MemSlot {
    pub offset: usize,
    pub owner_index: Option<usize>,
}
