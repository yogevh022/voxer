use crate::compute::chunk::VoxelChunkMeshMeta;
use crate::renderer::gpu::GPUChunkMeshEntry;

#[derive(Debug, Clone, Copy)]
pub enum MeshStateError {
    Missing,
    Empty,
    FailedAllocation,
}

#[derive(Debug, Clone)]
pub enum ChunkMeshState {
    Meshed(GPUChunkMeshEntry),
    Unmeshed(ChunkMeshUnmeshedEntry),
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkMeshUnmeshedEntry {
    negative_faces: u32,
    positive_faces: u32,
    pub total_faces: u32,
}

impl ChunkMeshUnmeshedEntry {
    pub fn has_faces(&self) -> bool {
        self.total_faces > 0
    }
}

impl ChunkMeshState {
    pub fn new_unmeshed(mesh_meta: VoxelChunkMeshMeta) -> Self {
        let negative_uvec3 = mesh_meta.negative_faces.as_uvec3();
        let positive_uvec3 = mesh_meta.positive_faces.as_uvec3();

        let total_faces = negative_uvec3.element_sum() + positive_uvec3.element_sum();

        let negative_faces: u32 =
            negative_uvec3.x | negative_uvec3.y << 10 | negative_uvec3.z << 20;

        let positive_faces: u32 =
            positive_uvec3.x | positive_uvec3.y << 10 | positive_uvec3.z << 20;

        let unmeshed_entry = ChunkMeshUnmeshedEntry {
            negative_faces,
            positive_faces,
            total_faces,
        };
        Self::Unmeshed(unmeshed_entry)
    }

    pub fn set_as_meshed(&mut self, index: u32, allocation: u32) {
        match self {
            ChunkMeshState::Unmeshed(unmeshed_entry) => {
                let mesh_entry = GPUChunkMeshEntry::new(
                    index,
                    unmeshed_entry.negative_faces,
                    unmeshed_entry.positive_faces,
                    allocation,
                );
                *self = ChunkMeshState::Meshed(mesh_entry);
            }
            ChunkMeshState::Meshed(_) => unreachable!(),
        }
    }

    pub fn entry_flagged_to_mesh(&self) -> GPUChunkMeshEntry {
        let mut entry = match self {
            Self::Meshed(entry) => entry.clone(),
            _ => unreachable!(),
        };
        // flag this entry to be meshed (meshing flag)
        entry.negative_face_count |= 1 << 31;
        entry
    }
}
