use crate::renderer::gpu::GPUChunkMeshEntry;
use crate::renderer::gpu::chunk_session_mesh_data::VoxelChunkMeshMeta;
use glam::UVec3;

#[derive(Debug, Clone)]
pub enum ChunkMeshEntry {
    GPU(GPUChunkMeshEntry),
    CPU(CPUChunkMeshEntry),
    Uninit,
}

// #[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPUChunkMeshEntry {
    pub face_count: u32,
    pub index: u32,
    negative_faces: UVec3,
    positive_faces: UVec3,
}

impl GPUChunkMeshEntry {
    // fixme should this be on declaration file?
    pub fn as_cpu(&self) -> CPUChunkMeshEntry {
        CPUChunkMeshEntry::new(
            self.index,
            unpack_face_count(self.negative_faces),
            unpack_face_count(self.positive_faces),
        )
    }
}

impl CPUChunkMeshEntry {
    fn new(index: u32, negative_faces: UVec3, positive_faces: UVec3) -> Self {
        Self {
            index,
            negative_faces,
            positive_faces,
            face_count: negative_faces.element_sum() + positive_faces.element_sum(),
        }
    }

    pub fn as_gpu(&self, index: u32, allocation: u32) -> GPUChunkMeshEntry {
        GPUChunkMeshEntry {
            index,
            negative_faces: pack_face_count(self.negative_faces),
            positive_faces: pack_face_count(self.positive_faces),
            face_alloc: allocation,
        }
    }
}

impl Default for ChunkMeshEntry {
    fn default() -> Self {
        Self::Uninit
    }
}

impl ChunkMeshEntry {
    pub fn new(mesh_meta: VoxelChunkMeshMeta, index: u32) -> Self {
        let negative_faces = mesh_meta.negative_faces.as_uvec3();
        let positive_faces = mesh_meta.positive_faces.as_uvec3();
        Self::CPU(CPUChunkMeshEntry::new(
            index,
            negative_faces,
            positive_faces,
        ))
    }
}

// impl ChunkMeshState {
//     pub fn meshing_flagged_entry(&self) -> GPUChunkMeshEntry {
//         let Self::Allocated(mut entry) = self.clone() else {
//             unreachable!(".meshing_flagged_entry() called on: {:?}", self);
//         };
//         // flag this entry to be meshed (meshing flag)
//         entry.negative_faces |= 1 << 31;
//         entry
//     }
// }

fn pack_face_count(face_count: UVec3) -> u32 {
    face_count.x | face_count.y << 10 | face_count.z << 20
}

fn unpack_face_count(face_count: u32) -> UVec3 {
    let x = face_count & 0x3FF;
    let y = (face_count >> 10) & 0x3FF;
    let z = face_count >> 20;
    UVec3::new(x, y, z)
}
