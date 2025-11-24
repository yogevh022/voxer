use crate::compute::num::MaybeUsize;

pub trait GpuIndexedItem: Clone {
    type WriteEntry: Clone;
    fn index(&self) -> usize;
    fn init(self) -> Self;
    fn reused(self) -> Self;
    fn write(self, index: usize) -> Self::WriteEntry;
}

pub struct VxGpuDeltaVec<T: GpuIndexedItem> {
    capacity: usize,
    buffer: Vec<T>,
    pub push_queue: Vec<T>,
    remove_queue: Vec<usize>,
    write_queue: Vec<T::WriteEntry>,
    id_to_index: Box<[MaybeUsize]>,
}

impl<T: GpuIndexedItem> VxGpuDeltaVec<T> {
    pub fn new(capacity: usize, queue_capacity: usize) -> Self {
        Self {
            capacity,
            buffer: Vec::with_capacity(capacity),
            remove_queue: Vec::with_capacity(queue_capacity),
            push_queue: Vec::with_capacity(queue_capacity),
            write_queue: Vec::with_capacity(queue_capacity),
            id_to_index: vec![MaybeUsize::default(); capacity].into_boxed_slice(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.push_queue.push(item);
    }

    pub fn remove(&mut self, id: usize) {
        self.remove_queue.push(id);
    }

    pub fn gpu_exists(&self, id: usize) -> bool {
        self.id_to_index[id].is_some()
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

        // asc sort remove_queue by buffer index (for non-overlapping swap removals)
        self.remove_queue
            .sort_by_key(|&idx| self.id_to_index[idx].unwrap());

        // record swap removals
        self.generate_remove_delta();

        // record pushes
        self.generate_push_delta();

        &self.write_queue
    }

    fn clear_index_mapping(&mut self, id: usize) {
        self.id_to_index[id] = MaybeUsize::default();
    }

    fn set_index_mapping(&mut self, id: usize, index: usize) {
        self.id_to_index[id] = MaybeUsize::new(index);
    }

    fn buffer_pop(&mut self, id: usize) {
        self.clear_index_mapping(id);
        self.buffer.pop().unwrap();
    }

    fn buffer_push(&mut self, item: T) {
        let buffer_index = self.buffer.len();
        let item_index = item.index();
        if self.gpu_exists(item_index) {
            return;
        }
        self.set_index_mapping(item_index, buffer_index);
        self.buffer.push(item.clone());
        self.write_queue.push(item.init().write(buffer_index));
    }

    fn buffer_swap_remove(&mut self, item_index: usize, buffer_index: usize) {
        let swap_item = self.buffer.pop().unwrap().reused();
        let swap_item_index = swap_item.index();
        self.clear_index_mapping(item_index);
        self.set_index_mapping(swap_item_index, buffer_index);
        self.buffer[buffer_index] = swap_item.clone();
        self.write_queue.push(swap_item.write(buffer_index));
    }

    fn generate_remove_delta(&mut self) {
        if self.remove_queue.is_empty() {
            return;
        }
        let mut drop_min = 0;
        let mut drop_max = self.remove_queue.len();
        let mut buff_max = self.buffer.len();
        while drop_min < drop_max {
            let drop_min_id = self.remove_queue[drop_min];
            let drop_min_index = self.id_to_index[drop_min_id].unwrap();
            if drop_min_index >= buff_max {
                break;
            }
            let drop_max_id = self.remove_queue[drop_max - 1];
            let drop_max_index = self.id_to_index[drop_max_id].unwrap();

            if drop_max_index == buff_max - 1 {
                self.buffer_pop(drop_max_id);
                drop_max -= 1;
            } else {
                self.buffer_swap_remove(drop_min_id, drop_min_index);
                drop_min += 1;
            }
            buff_max -= 1;
        }
        self.remove_queue.clear();
    }

    fn generate_push_delta(&mut self) {
        for _ in 0..self.push_queue.len() {
            let item = unsafe { self.push_queue.pop().unwrap_unchecked() };
            self.buffer_push(item);
        }
    }
}
