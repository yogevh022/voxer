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
        adjacent_x(&packed_blocks, &mut xa, &mut xb, i);
        adjacent_y(&packed_blocks, &mut ya, &mut yb, i);
        adjacent_z(&xa, &mut zb);

        result_layers[i] = compute::array::xor(&xa, &xb);
        result_layers[CHUNK_DIM + i] = compute::array::xor(&ya, &yb);
        result_layers[CHUNK_DIM + CHUNK_DIM + i] = compute::array::xor(&xa, &zb);
    }
    adjacent_y(&packed_blocks, &mut ya, &mut yb, CHUNK_DIM - 1);
    adjacent_z(&xb, &mut zb);
    result_layers[CHUNK_DIM - 1] = compute::array::xor(&xb, &TRANSPARENT_LAYER_BITS);
    result_layers[CHUNK_DIM + (CHUNK_DIM - 1)] = compute::array::xor(&ya, &yb);
    result_layers[CHUNK_DIM + CHUNK_DIM + (CHUNK_DIM - 1)] = compute::array::xor(&xb, &zb);
    result
}


#[inline(always)]
fn adjacent_x(
    packed_blocks: &[u16; CHUNK_SLICE],
    xa: &mut [u16; CHUNK_DIM],
    xb: &mut [u16; CHUNK_DIM],
    x: usize
) {
    *xa = packed_blocks[x * CHUNK_DIM..(x + 1) * CHUNK_DIM]
        .try_into()
        .unwrap();
    *xb = packed_blocks[(x + 1) * CHUNK_DIM..(x + 2) * CHUNK_DIM]
        .try_into()
        .unwrap();
}

#[inline(always)]
fn adjacent_y(
    packed_blocks: &[u16; CHUNK_SLICE],
    ya: &mut [u16; CHUNK_DIM],
    yb: &mut [u16; CHUNK_DIM],
    x: usize
) {
    for j in 0..CHUNK_DIM - 1 {
        ya[j] = packed_blocks[(x * CHUNK_DIM) + j];
        yb[j] = packed_blocks[(x * CHUNK_DIM) + j + 1];
    }
    ya[CHUNK_DIM - 1] = packed_blocks[(x * CHUNK_DIM) + (CHUNK_DIM - 1)];
    yb[CHUNK_DIM - 1] = 0u16;
}

#[inline(always)]
fn adjacent_z(
    xa: &[u16; CHUNK_DIM],
    zb: &mut [u16; CHUNK_DIM],
) {
    *zb = array::from_fn(|i| xa[i] >> 1);
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
