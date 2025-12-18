use crate::renderer::gpu::{GPUChunkMeshEntry, GPUVoxelChunkHeader};

#[derive(Debug, Clone)]
pub struct ChunkMeshEntry {
    header: GPUVoxelChunkHeader,
    faces_count: u32,
    face_alloc: Option<u32>,
}

impl ChunkMeshEntry {
    pub fn new(header: GPUVoxelChunkHeader, faces_count: u32) -> Self {
        Self {
            header,
            faces_count,
            face_alloc: None,
        }
    }

    #[inline(always)]
    pub fn take_face_alloc(&mut self) -> Option<u32> {
        self.face_alloc.take()
    }

    #[inline(always)]
    pub fn set_face_alloc(&mut self, face_alloc: u32) {
        self.face_alloc = Some(face_alloc);
    }

    #[inline(always)]
    pub fn is_allocated(&self) -> bool {
        self.face_alloc.is_some()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.faces_count == 0
    }

    #[inline(always)]
    pub fn index(&self) -> u32 {
        self.header.index
    }

    #[inline(always)]
    pub fn faces_count(&self) -> u32 {
        self.faces_count
    }

    #[inline(always)]
    pub fn gpu_entry(&self) -> GPUChunkMeshEntry {
        GPUChunkMeshEntry {
            index: self.header.index,
            face_alloc: self.face_alloc.unwrap(),
        }
    }
}
