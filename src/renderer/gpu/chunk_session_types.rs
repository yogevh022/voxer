use crate::renderer::gpu::{GPUChunkMeshEntry, GPUVoxelChunkHeader};
use glam::{IVec3, UVec3};

#[derive(Debug, Clone)]
pub struct ChunkMeshEntry {
    pub header: GPUVoxelChunkHeader,
    pub faces_count: u32,
    pub face_alloc: Option<u32>,
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

//
// #[derive(Debug, Clone)]
// pub enum ChunkMeshEntry {
//     GPU(GPUChunkMeshEntry),
//     CPU(CPUChunkMeshEntry),
//     Uninit,
// }

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct CPUChunkMeshEntry {
//     pub face_count: u32,
//     pub index: u32,
//     faces_positive: UVec3,
//     faces_negative: UVec3,
// }
//
// impl GPUChunkMeshEntry {
//     // fixme should this be on declaration file?
//     pub fn as_cpu(&self) -> CPUChunkMeshEntry {
//         CPUChunkMeshEntry::new(
//             self.index,
//             unpack_face_count(self.negative_faces),
//             unpack_face_count(self.positive_faces),
//         )
//     }
// }
//
// impl CPUChunkMeshEntry {
//     fn new(index: u32, negative_faces: UVec3, positive_faces: UVec3) -> Self {
//         Self {
//             index,
//             faces_negative: negative_faces,
//             faces_positive: positive_faces,
//             face_count: negative_faces.element_sum() + positive_faces.element_sum(),
//         }
//     }
//
//     pub fn as_gpu(&self, index: u32, allocation: u32) -> GPUChunkMeshEntry {
//         GPUChunkMeshEntry {
//             index,
//             negative_faces: pack_face_count(self.faces_negative),
//             positive_faces: pack_face_count(self.faces_positive),
//             face_alloc: allocation,
//         }
//     }
// }

// impl Default for ChunkMeshEntry {
//     fn default() -> Self {
//         Self::Uninit
//     }
// }
//
// impl ChunkMeshEntry {
//     pub fn new(mesh_meta: VoxelChunkMeshMeta, index: u32) -> Self {
//         let negative_faces = mesh_meta.faces_negative.as_uvec3();
//         let positive_faces = mesh_meta.faces_positive.as_uvec3();
//         Self::CPU(CPUChunkMeshEntry::new(
//             index,
//             negative_faces,
//             positive_faces,
//         ))
//     }
// }

// fn pack_face_count(face_count: UVec3) -> u32 {
//     face_count.x | face_count.y << 12 | face_count.z << 24
// }
//
// fn unpack_face_count(face_count: u32) -> UVec3 {
//     let x = face_count & 0x3FF;
//     let y = (face_count >> 10) & 0x3FF;
//     let z = face_count >> 20;
//     UVec3::new(x, y, z)
// }
