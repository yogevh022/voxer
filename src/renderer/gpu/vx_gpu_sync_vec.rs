use crate::compute::num::MaybeUsize;

pub trait GpuIndexedItem: Copy {
    type WriteEntry: Copy;
    fn index(&self) -> usize;
    fn init(self) -> Self;
    fn reused(self) -> Self;
    fn write(self, index: usize) -> Self::WriteEntry;
}

pub struct VxGpuSyncVec<T: GpuIndexedItem> {
    capacity: usize,
    buffer: Vec<T>,
    push_queue: Vec<T>,
    remove_queue: Vec<usize>,
    write_queue: Vec<T::WriteEntry>,
    buffer_mapping: Box<[MaybeUsize]>,
}

impl<T: GpuIndexedItem> VxGpuSyncVec<T> {
    pub fn new(capacity: usize, queue_capacity: usize) -> Self {
        Self {
            capacity,
            buffer: Vec::with_capacity(capacity),
            remove_queue: Vec::with_capacity(queue_capacity),
            push_queue: Vec::with_capacity(queue_capacity),
            write_queue: Vec::with_capacity(queue_capacity),
            buffer_mapping: vec![MaybeUsize::default(); capacity].into_boxed_slice(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.push_queue.push(item);
    }

    pub fn remove(&mut self, id: usize) {
        let index = self.buffer_mapping[id].take().unwrap();
        self.remove_queue.push(index);
    }

    pub fn cpu_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn cpu_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn cpu_dirty(&self) -> bool {
        !self.push_queue.is_empty() || !self.remove_queue.is_empty()
    }

    pub fn sync_delta(&mut self) -> &[T::WriteEntry] {
        // clear write queue
        self.write_queue.clear();

        // asc sort remove_queue (by index, for non-overlapping swap removals)
        self.remove_queue.sort();

        self.cpu_write_delta();

        &self.write_queue
    }

    fn set_index_mapping(&mut self, id: usize, index: usize) {
        self.buffer_mapping[id] = MaybeUsize::new(index);
    }

    fn buffer_exists(&self, id: usize) -> bool {
        self.buffer_mapping[id].is_some()
    }

    fn buffer_pop(&mut self) -> T {
        self.buffer.pop().unwrap()
    }

    fn buffer_push(&mut self, item: T) {
        let buffer_index = self.buffer.len();
        self.set_index_mapping(item.index(), buffer_index);
        self.write_queue.push(item.write(buffer_index));
        self.buffer.push(item);
    }

    fn buffer_insert(&mut self, item: T, buffer_index: usize) {
        self.set_index_mapping(item.index(), buffer_index);
        self.write_queue.push(item.write(buffer_index));
        self.buffer[buffer_index] = item;
    }

    fn cpu_write_delta(&mut self) {
        let mut drop_min = 0;
        let mut drop_max = self.remove_queue.len();

        for i in 0..self.push_queue.len() {
            let item = unsafe { *self.push_queue.get_unchecked(i) }.init();
            debug_assert!(!self.buffer_exists(item.index()));
            if drop_min < drop_max {
                let remove_index = self.remove_queue[drop_min];
                self.buffer_insert(item, remove_index);
                drop_min += 1;
            } else {
                self.buffer_push(item);
            }
        }
        self.push_queue.clear();

        let mut buff_max = self.buffer.len();
        while drop_min < drop_max {
            let drop_min_index = self.remove_queue[drop_min];
            if drop_min_index >= buff_max {
                break;
            }
            let drop_max_index = self.remove_queue[drop_max - 1];

            if drop_max_index == buff_max - 1 {
                self.buffer_pop();
                drop_max -= 1;
            } else {
                let swap = self.buffer_pop().reused();
                self.buffer_insert(swap, drop_min_index);
                drop_min += 1;
            }
            buff_max -= 1;
        }
        self.remove_queue.clear();
    }
}
