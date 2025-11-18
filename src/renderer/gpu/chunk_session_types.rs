use crate::renderer::gpu::GPUChunkMeshEntry;
use crate::renderer::gpu::chunk_session_mesh_data::VoxelChunkMeshMeta;
use glam::UVec3;

#[derive(Debug, Clone)]
pub enum ChunkMeshState {
    Allocated(GPUChunkMeshEntry),
    Unallocated(ChunkUnmeshedEntry),
    Uninitialized,
    AllocatedEmpty,
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkUnmeshedEntry {
    negative_faces: u32,
    positive_faces: u32,
}

impl ChunkUnmeshedEntry {
    pub fn face_count(&self) -> u32 {
        unpack_face_count(self.negative_faces).element_sum()
            + unpack_face_count(self.positive_faces).element_sum()
    }
}

impl ChunkMeshState {
    pub fn new(mesh_meta: VoxelChunkMeshMeta) -> Self {
        Self::Unallocated(ChunkUnmeshedEntry {
            negative_faces: pack_face_count(mesh_meta.negative_faces.as_uvec3()),
            positive_faces: pack_face_count(mesh_meta.positive_faces.as_uvec3()),
        })
    }

    pub fn set_allocated(&mut self, index: u32, allocation: u32) {
        let Self::Unallocated(unmeshed_entry) = self else {
            unreachable!(".set_meshed() called on non Unallocated state: {:?}", self);
        };
        *self = Self::Allocated(GPUChunkMeshEntry {
            index,
            negative_faces: unmeshed_entry.negative_faces,
            positive_faces: unmeshed_entry.positive_faces,
            face_alloc: allocation,
        });
    }

    pub fn set_unallocated(&mut self) {
        let unmeshed_entry = match self {
            Self::Allocated(entry) => ChunkUnmeshedEntry {
                negative_faces: entry.negative_faces,
                positive_faces: entry.positive_faces,
            },
            Self::AllocatedEmpty => ChunkUnmeshedEntry {
                negative_faces: 0,
                positive_faces: 0,
            },
            _ => unreachable!(".set_unmeshed() called on Unallocated: {:?}", self),
        };
        *self = Self::Unallocated(unmeshed_entry);
    }

    pub fn set_empty(&mut self) {
        *self = Self::AllocatedEmpty;
    }

    pub fn meshing_flagged_entry(&self) -> GPUChunkMeshEntry {
        let Self::Allocated(mut entry) = self.clone() else {
            unreachable!(
                ".meshing_flagged_entry() called on non Allocated state: {:?}",
                self
            );
        };
        // flag this entry to be meshed (meshing flag)
        entry.negative_faces |= 1 << 31;
        entry
    }
}

fn pack_face_count(face_count: UVec3) -> u32 {
    face_count.x | face_count.y << 10 | face_count.z << 20
}

fn unpack_face_count(face_count: u32) -> UVec3 {
    let x = face_count & 0x3FF;
    let y = (face_count >> 10) & 0x3FF;
    let z = face_count >> 20;
    UVec3::new(x, y, z)
}
