use crate::renderer::gpu::{GPUChunkMeshEntry, GPUVoxelChunkHeader};

#[derive(Debug, Clone)]
pub struct ChunkMeshEntry {
    header: GPUVoxelChunkHeader,
    face_alloc: Option<u32>,
    visible: bool,
}

impl ChunkMeshEntry {
    pub fn new(header: GPUVoxelChunkHeader, face_alloc: Option<u32>) -> Self {
        Self {
            header,
            face_alloc,
            visible: false,
        }
    }

    #[inline(always)]
    pub fn index(&self) -> u32 {
        self.header.index
    }

    #[inline(always)]
    pub fn face_alloc(&self) -> u32 {
        self.face_alloc.unwrap()
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
        self.face_alloc.is_none()
    }

    #[inline(always)]
    pub fn gpu_entry(&self) -> GPUChunkMeshEntry {
        GPUChunkMeshEntry {
            index: self.header.index,
            face_alloc: self.face_alloc.unwrap(),
        }
    }
}
