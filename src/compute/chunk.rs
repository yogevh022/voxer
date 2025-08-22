use crate::compute;
use crate::compute::array::Array3D;
use crate::world::types::{Block, BlockBytewise, CHUNK_DIM, CHUNK_SLICE, ChunkBlocks};
use glam::IVec3;
use std::array;

pub const TRANSPARENT_LAYER_BITS: [u16; CHUNK_DIM] = [0u16; CHUNK_DIM];

pub fn face_count(blocks: &ChunkBlocks) -> usize {
    let packed_blocks = pack_solid_blocks(blocks);

    let faces = faces(packed_blocks);
    faces.iter().map(|b| b.count_ones() as usize).sum::<usize>()
}

pub fn position_to_id(position: IVec3) -> u128 {
    ((position.x as u128) << 64) | ((position.y as u128) << 32) | (position.z as u128)
}

fn faces(packed_blocks: [u16; CHUNK_SLICE]) -> [u16; CHUNK_SLICE * 3] {
    let mut result = [0u16; CHUNK_SLICE * 3];
    let result_layers: &mut [[u16; CHUNK_DIM]; CHUNK_DIM * 3] =
        unsafe { &mut *(result.as_mut_ptr() as *mut [[u16; CHUNK_DIM]; CHUNK_DIM * 3]) };

    let mut xa = [0u16; CHUNK_DIM];
    let mut xb = [0u16; CHUNK_DIM];
    let mut ya = [0u16; CHUNK_DIM];
    let mut yb = [0u16; CHUNK_DIM];
    let mut zb = [0u16; CHUNK_DIM];

    for i in 0..CHUNK_DIM - 1 {
        // x faces
        xa = packed_blocks[i * CHUNK_DIM..(i + 1) * CHUNK_DIM]
            .try_into()
            .unwrap();
        xb = packed_blocks[(i + 1) * CHUNK_DIM..(i + 2) * CHUNK_DIM]
            .try_into()
            .unwrap();
        // y faces
        for j in 0..CHUNK_DIM {
            ya[j] = packed_blocks[j + (i * CHUNK_DIM)];
            yb[j] = packed_blocks[j + ((i + 1) * CHUNK_DIM)];
        }
        // x faces
        zb = array::from_fn(|i| xa[i] >> 1);

        result_layers[i] = compute::array::xor(&xa, &xb);
        result_layers[CHUNK_DIM + i] = compute::array::xor(&ya, &yb);
        result_layers[CHUNK_DIM + CHUNK_DIM + i] = compute::array::xor(&xa, &zb);
    }
    result_layers[CHUNK_DIM - 1] = compute::array::xor(&xb, &TRANSPARENT_LAYER_BITS);
    result_layers[CHUNK_DIM + (CHUNK_DIM - 1)] = compute::array::xor(&yb, &TRANSPARENT_LAYER_BITS);
    zb = array::from_fn(|i| xb[i] >> 1);
    result_layers[CHUNK_DIM + CHUNK_DIM + (CHUNK_DIM - 1)] = compute::array::xor(&xb, &zb);
    result
}

fn pack_solid_blocks(
    blocks: &Array3D<Block, CHUNK_DIM, CHUNK_DIM, CHUNK_DIM>,
) -> [u16; CHUNK_SLICE] {
    // packs chunk blocks into a bit (u16) array, 1 for solid 0 for transparent
    // Array3D<Block, CHUNK_DIM> -> Array1D<u16, CHUNK_SLICE>
    // compiles to SIMD
    let mut bytes = [0u16; CHUNK_SLICE];

    for (byte_idx, row) in blocks.iter().flatten().enumerate() {
        let mut bits = 0u16;
        for (i, b) in row.iter().enumerate() {
            bits |= (!b.is_transparent() as u16) << i;
        }
        bytes[byte_idx] = bits;
    }

    bytes
}
