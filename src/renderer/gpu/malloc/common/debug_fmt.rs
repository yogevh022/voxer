use crate::compute;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DebugFmtMemSlot {
    pub offset: usize,
    pub size: usize,
}

impl Ord for DebugFmtMemSlot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl PartialOrd for DebugFmtMemSlot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn push_bar_char(s: &mut String, char: char, size: &mut usize, row_size: usize) {
    if *size % row_size == 0 {
        s.push('\n');
    }
    s.push(char);
    *size += 1;
}

pub fn malloc_fmt<'a>(
    f: &mut std::fmt::Formatter<'_>,
    total_size: usize,
    num_allocated: usize,
    num_free: usize,
    sorted_free_iter: impl Iterator<Item = &'a DebugFmtMemSlot>,
) -> std::fmt::Result {
    const ROWS: usize = 10;
    let resolution = total_size / (compute::KIB >> 1);
    let row_size = (total_size / resolution) / ROWS;
    let mut debug_bar = String::with_capacity((total_size / resolution) + ROWS);
    let mut bar_i = 0;
    let mut total_free = 0usize;
    for free_slot in sorted_free_iter {
        if free_slot.offset / resolution != bar_i {
            for _ in 0..(free_slot.offset / resolution) - bar_i {
                push_bar_char(&mut debug_bar, '#', &mut bar_i, row_size);
            }
        }
        for _ in 0..free_slot.size / resolution {
            push_bar_char(&mut debug_bar, '-', &mut bar_i, row_size);
        }
        total_free += free_slot.size;
    }
    write!(
        f,
        "~used: {:>6} / {:>6}\nalloc: {:>6} ~avg: {:>6}\nfree:  {:>6} ~avg: {:>6}\n{}",
        total_size - total_free,
        total_size,
        num_allocated,
        (total_size - total_free) / num_allocated.max(1),
        num_free,
        (total_size) / num_free.max(1),
        debug_bar
    )
}
