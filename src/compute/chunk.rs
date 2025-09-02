use crate::compute;
use crate::compute::array::Array3D;
use crate::world::types::{
    Block, BlockBytewise, CHUNK_DIM, CHUNK_SLICE, ChunkBlocks, ChunkRelevantBlocks,
};
use std::array;

pub const TRANSPARENT_LAYER_BITS: [u16; CHUNK_DIM] = [0u16; CHUNK_DIM];
pub const TRANSPARENT_LAYER_BLOCKS: [[Block; CHUNK_DIM]; CHUNK_DIM] =
    [[Block { value: 0 }; CHUNK_DIM]; CHUNK_DIM];

pub fn face_count(chunk_rel: &ChunkRelevantBlocks) -> usize {
    let packed_blocks = pack_solid_blocks(&chunk_rel.chunk.blocks);
    let packed_adjacent_blocks = pack_solid_blocks(&chunk_rel.adjacent_blocks);

    let faces = faces(packed_blocks, packed_adjacent_blocks);
    faces.iter().map(|b| b.count_ones() as usize).sum::<usize>()
}

fn faces(
    packed_blocks: [u16; CHUNK_SLICE],
    packed_adjacent_blocks: [u16; CHUNK_DIM * 3],
) -> [u16; CHUNK_SLICE * 3] {
    let mut result = [0u16; CHUNK_SLICE * 3];
    let result_layers: &mut [[u16; CHUNK_DIM]; CHUNK_DIM * 3] =
        unsafe { &mut *(result.as_mut_ptr() as *mut [[u16; CHUNK_DIM]; CHUNK_DIM * 3]) };

    let mut za = [0u16; CHUNK_DIM];
    let mut zb = [0u16; CHUNK_DIM];
    let mut ya = [0u16; CHUNK_DIM];
    let mut yb = [0u16; CHUNK_DIM];
    let mut xb = [0u16; CHUNK_DIM];

    for i in 0..CHUNK_DIM - 1 {
        adjacent_z(&packed_blocks, &mut za, &mut zb, i);
        adjacent_y(
            &packed_blocks,
            packed_adjacent_blocks[CHUNK_DIM + i],
            &mut ya,
            &mut yb,
            i,
        );
        adjacent_x(packed_adjacent_blocks[(CHUNK_DIM * 2) + i], &za, &mut xb);

        result_layers[i] = compute::array::xor(&za, &zb);
        result_layers[CHUNK_DIM + i] = compute::array::xor(&ya, &yb);
        result_layers[CHUNK_DIM + CHUNK_DIM + i] = compute::array::xor(&za, &xb);
    }
    adjacent_y(
        &packed_blocks,
        packed_adjacent_blocks[CHUNK_DIM + CHUNK_DIM - 1],
        &mut ya,
        &mut yb,
        CHUNK_DIM - 1,
    );
    adjacent_x(
        packed_adjacent_blocks[(CHUNK_DIM * 2) + CHUNK_DIM - 1],
        &zb,
        &mut xb,
    );
    result_layers[CHUNK_DIM - 1] = compute::array::xor(
        &zb,
        &packed_adjacent_blocks[0..CHUNK_DIM].try_into().unwrap(),
    );
    result_layers[CHUNK_DIM + (CHUNK_DIM - 1)] = compute::array::xor(&ya, &yb);
    result_layers[CHUNK_DIM + CHUNK_DIM + (CHUNK_DIM - 1)] = compute::array::xor(&zb, &xb);
    result
}

#[inline(always)]
fn adjacent_z(
    packed_blocks: &[u16; CHUNK_SLICE],
    xa: &mut [u16; CHUNK_DIM],
    xb: &mut [u16; CHUNK_DIM],
    x: usize,
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
    packed_adjacent_blocks: u16,
    ya: &mut [u16; CHUNK_DIM],
    yb: &mut [u16; CHUNK_DIM],
    x: usize,
) {
    for j in 0..CHUNK_DIM - 1 {
        ya[j] = packed_blocks[(x * CHUNK_DIM) + j];
        yb[j] = packed_blocks[(x * CHUNK_DIM) + j + 1];
    }
    ya[CHUNK_DIM - 1] = packed_blocks[(x * CHUNK_DIM) + (CHUNK_DIM - 1)];
    yb[CHUNK_DIM - 1] = packed_adjacent_blocks;
}

#[inline(always)]
fn adjacent_x(packed_adjacent_blocks: u16, xa: &[u16; CHUNK_DIM], zb: &mut [u16; CHUNK_DIM]) {
    *zb = array::from_fn(|i| ((packed_adjacent_blocks & (1 << i)) << 15) | (xa[i] >> 1));
}

fn pack_solid_blocks<const X: usize, const Y: usize, const Z: usize, const YZ: usize>(
    blocks: &Array3D<Block, X, Y, Z>,
) -> [u16; YZ] {
    // packs chunk blocks into a bit (u16) array, 1 for solid 0 for transparent
    // Array3D<Block, CHUNK_DIM> -> Array1D<u16, CHUNK_SLICE>
    // compiles to SIMD
    let mut bytes = [0u16; YZ];

    for (byte_idx, row) in blocks.iter().flatten().enumerate() {
        let mut bits = 0u16;
        for (i, b) in row.iter().enumerate() {
            bits |= (!b.is_transparent() as u16) << i;
        }
        bytes[byte_idx] = bits;
    }

    bytes
}

pub fn get_mx_layer(blocks: &ChunkBlocks) -> [[Block; CHUNK_DIM]; CHUNK_DIM] {
    blocks[0]
}

pub fn get_my_layer(blocks: &ChunkBlocks) -> [[Block; CHUNK_DIM]; CHUNK_DIM] {
    array::from_fn(|i| blocks[i][0])
}

pub fn get_mz_layer(blocks: &ChunkBlocks) -> [[Block; CHUNK_DIM]; CHUNK_DIM] {
    array::from_fn(|x| array::from_fn(|y| blocks[x][y][0]))
}
