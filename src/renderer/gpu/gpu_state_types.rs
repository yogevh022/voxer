use crate::renderer::gpu::GPUChunkMeshEntry;

#[derive(Debug, Clone)]
pub enum ChunkMeshState {
    Meshed(GPUChunkMeshEntry),
    Unmeshed(u32),
}

impl ChunkMeshState {
    pub fn set_meshed(&mut self, index: u32, allocation: u32) {
        match self {
            ChunkMeshState::Unmeshed(face_count) => {
                let mesh_entry = GPUChunkMeshEntry::new(index, *face_count, allocation);
                *self = ChunkMeshState::Meshed(mesh_entry);
            }
            ChunkMeshState::Meshed(_) => unreachable!(),
        }
    }

    pub fn mesh_entry(&self) -> &GPUChunkMeshEntry {
        match self {
            ChunkMeshState::Meshed(entry) => entry,
            ChunkMeshState::Unmeshed(_) => unreachable!(),
        }
    }
}

type GPUListSetLenFunc<T> = fn(&mut T, u32);
pub struct GPUList<T> {
    data: Vec<T>,
    set_len_func: GPUListSetLenFunc<T>,
    can_write: bool,
}

impl<T> GPUList<T> {
    pub fn new(set_len_func: GPUListSetLenFunc<T>) -> Self {
        Self::with_capacity(1, set_len_func)
    }

    pub fn with_capacity(capacity: usize, set_len_func: GPUListSetLenFunc<T>) -> Self {
        debug_assert_ne!(capacity, 0);
        let mut gpu_list = Self {
            data: Vec::with_capacity(capacity),
            set_len_func,
            can_write: false,
        };
        gpu_list.clear();
        gpu_list.done();
        gpu_list
    }

    pub fn push(&mut self, item: T) {
        debug_assert!(self.can_write);
        self.data.push(item);
    }

    pub fn clear(&mut self) {
        self.data.clear();
        unsafe { self.data.set_len(1) };
        self.can_write = true;
    }

    pub fn len(&self) -> usize {
        self.data.len() - 1
    }

    pub fn done(&mut self) {
        let list_len = self.data.len() - 1;
        (self.set_len_func)(&mut self.data[0], list_len as u32);
        self.can_write = false;
    }

    pub fn inner_slice(&self) -> &[T] {
        debug_assert!(!self.can_write);
        &self.data
    }
}