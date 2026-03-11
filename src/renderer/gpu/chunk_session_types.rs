use crate::renderer::gpu::{GPUChunkMeshEntry, GPUVoxelChunkHeader};

#[derive(Debug, Clone)]
pub struct ChunkMeshEntry {
    gpu_entry: GPUChunkMeshEntry,
    empty: bool,
    visible: bool,
}

impl ChunkMeshEntry {
    pub fn new(index: u32, face_alloc: u32) -> Self {
        Self {
            gpu_entry: GPUChunkMeshEntry { index, face_alloc },
            empty: false,
            visible: false,
        }
    }

    pub fn new_empty(index: u32) -> Self {
        Self {
            gpu_entry: GPUChunkMeshEntry {
                index,
                face_alloc: 0,
            },
            empty: true,
            visible: false,
        }
    }

    #[inline(always)]
    pub fn index(&self) -> u32 {
        self.gpu_entry.index
    }

    #[inline(always)]
    pub fn face_alloc(&self) -> u32 {
        self.gpu_entry.face_alloc
    }

    #[inline(always)]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    #[inline(always)]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.empty
    }

    #[inline(always)]
    pub fn gpu_entry(&self) -> GPUChunkMeshEntry {
        self.gpu_entry
    }
}
