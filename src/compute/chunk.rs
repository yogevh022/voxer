use crate::compute;
use crate::compute::array::Array3D;
use crate::worldgen::types::{Block, CHUNK_DIM, CHUNK_SLICE, CHUNK_VOLUME, Chunk, ChunkBlocks};
use simdeez::Simd;
use simdeez::avx2::Avx2;
use simdeez::scalar::Scalar;
use simdeez::simd_runtime_generate;
use simdeez::sse41::Sse2;
use simdeez::sse41::Sse41;
use std::ops::{Deref, DerefMut};

pub type ChunkBits = [u16; CHUNK_DIM * CHUNK_DIM];
type ChunkBits2D = [[u16; CHUNK_DIM]; CHUNK_DIM];
type ChunkLayerBits = [u16; CHUNK_DIM];

pub const OPAQUE_LAYER: [Block; CHUNK_SLICE] = [Block(1 << 15); CHUNK_SLICE];

pub fn chunk_face_count(chunk: &Chunk) -> usize {
    let x_blocks = &chunk.blocks;
    let x_faces = faces_on_x(x_blocks);

    let y_blocks = compute::array::rotated_z(x_blocks);
    let y_faces = faces_on_x(&y_blocks);

    let z_blocks = compute::array::rotated_y(x_blocks);
    let z_faces = faces_on_x(&z_blocks);

    x_faces.iter().map(|b| (b >> 15) as usize).sum::<usize>()
        + y_faces.iter().map(|b| (b >> 15) as usize).sum::<usize>()
        + z_faces.iter().map(|b| (b >> 15) as usize).sum::<usize>()
}

fn faces_on_x(blocks: &ChunkBlocks) -> [u16; CHUNK_VOLUME] {
    let yz_layers: &[[Block; CHUNK_SLICE]; CHUNK_DIM] =
        unsafe { &*(blocks.as_ptr() as *const [[Block; CHUNK_SLICE]; CHUNK_DIM]) };

    let mut result = [0u16; CHUNK_VOLUME];
    let result_layers: &mut [[u16; CHUNK_SLICE]; CHUNK_DIM] =
        unsafe { &mut *(result.as_mut_ptr() as *mut [[u16; CHUNK_SLICE]; CHUNK_DIM]) };

    for i in 0..CHUNK_DIM - 1 {
        let layer_a = &yz_layers[i];
        let layer_b = &yz_layers[i + 1];
        result_layers[i] = layer_face_data_runtime_select(layer_a, layer_b);
    }
    let layer_a = &yz_layers[CHUNK_DIM - 1];
    result_layers[CHUNK_DIM - 1] = layer_face_data_runtime_select(layer_a, &OPAQUE_LAYER);

    result
}

simd_runtime_generate!(
    fn layer_face_data(
        a_u16: &[Block; CHUNK_SLICE],
        b_u16: &[Block; CHUNK_SLICE],
    ) -> [u16; CHUNK_SLICE] {
        let a_i32: &[i32; CHUNK_SLICE / 2] = unsafe { std::mem::transmute(a_u16) };
        let b_i32: &[i32; CHUNK_SLICE / 2] = unsafe { std::mem::transmute(b_u16) };

        let mut faces = [0i32; CHUNK_SLICE / 2];
        let s_chunks = CHUNK_SLICE / S::VI16_WIDTH;
        unsafe {
            for i in 0..s_chunks {
                let va = S::loadu_epi32(&a_i32[i * S::VF32_WIDTH]);
                let vb = S::loadu_epi32(&b_i32[i * S::VF32_WIDTH]);

                let face_bits = S::xor_epi32(va, vb);

                S::storeu_epi32(&mut faces[i * S::VF32_WIDTH], face_bits);
            }
            std::mem::transmute(faces)
        }
    }
);
