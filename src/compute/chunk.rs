use crate::compute;
use crate::compute::array::Array3D;
use crate::compute::bytes::bit_at;
use crate::world::types::{VoxelBlock, BlockBytewise, CHUNK_DIM, CHUNK_SLICE, Chunk, ChunkBlocks, ChunkAdjacentBlocks};
use glam::IVec3;
use rustc_hash::FxHashMap;
use std::array;

pub const TRANSPARENT_LAYER_BITS: [u16; CHUNK_DIM] = [0u16; CHUNK_DIM];
pub const TRANSPARENT_LAYER_BLOCKS: [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] =
    [[VoxelBlock { value: 0 }; CHUNK_DIM]; CHUNK_DIM];

pub fn face_count(blocks: &ChunkBlocks, adjacent_blocks: &ChunkAdjacentBlocks) -> usize {
    let next_adj_blocks = unsafe {
        *adjacent_blocks.as_ptr().cast::<Array3D<VoxelBlock, 3, CHUNK_DIM, CHUNK_DIM>>()
    };
    let packed_blocks = pack_solid_blocks(blocks);
    let packed_adjacent_blocks = pack_solid_blocks(&next_adj_blocks);

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

    let mut xa = [0u16; CHUNK_DIM];
    let mut xb = [0u16; CHUNK_DIM];
    let mut ya = [0u16; CHUNK_DIM];
    let mut yb = [0u16; CHUNK_DIM];
    let mut zb = [0u16; CHUNK_DIM];

    for i in 0..CHUNK_DIM - 1 {
        adjacent_x(&packed_blocks, &mut xa, &mut xb, i);
        adjacent_y(
            &packed_blocks,
            packed_adjacent_blocks[CHUNK_DIM + i],
            &mut ya,
            &mut yb,
            i,
        );
        adjacent_z(
            packed_adjacent_blocks[CHUNK_DIM + CHUNK_DIM + i],
            &xa,
            &mut zb,
        );

        result_layers[i] = compute::array::array_xor(&xa, &xb);
        result_layers[CHUNK_DIM + i] = compute::array::array_xor(&ya, &yb);
        result_layers[CHUNK_DIM + CHUNK_DIM + i] = compute::array::array_xor(&xa, &zb);
    }
    adjacent_y(
        &packed_blocks,
        packed_adjacent_blocks[CHUNK_DIM + CHUNK_DIM - 1],
        &mut ya,
        &mut yb,
        CHUNK_DIM - 1,
    );
    adjacent_z(
        packed_adjacent_blocks[CHUNK_DIM + CHUNK_DIM + CHUNK_DIM - 1],
        &xb,
        &mut zb,
    );
    result_layers[CHUNK_DIM - 1] = compute::array::array_xor(
        &xb,
        &packed_adjacent_blocks[0..CHUNK_DIM].try_into().unwrap(),
    );
    result_layers[CHUNK_DIM + CHUNK_DIM - 1] = compute::array::array_xor(&ya, &yb);
    result_layers[CHUNK_DIM + CHUNK_DIM + CHUNK_DIM - 1] = compute::array::array_xor(&xb, &zb);
    result
}

#[inline(always)]
fn adjacent_x(
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
fn adjacent_z(packed_adjacent_blocks: u16, xa: &[u16; CHUNK_DIM], zb: &mut [u16; CHUNK_DIM]) {
    *zb = array::from_fn(|i| bit_at(packed_adjacent_blocks, i) << 15 | (xa[i] >> 1));
}

fn pack_solid_blocks<const X: usize, const Y: usize, const Z: usize, const XY: usize>(
    blocks: &Array3D<VoxelBlock, X, Y, Z>,
) -> [u16; XY] {
    // packs chunk blocks into a bit (u16) array, 1 for solid 0 for transparent
    // Array3D<Block, X, Y, Z> -> Array1D<u16, XY>
    // compiles to SIMD
    let mut bytes = [0u16; XY];

    for (byte_idx, row) in blocks.iter().flatten().enumerate() {
        let mut bits = 0u16;
        for (i, b) in row.iter().enumerate() {
            bits |= (!b.is_transparent() as u16) << i;
        }
        bytes[byte_idx] = bits;
    }

    bytes
}

fn get_mx_layer(blocks: &ChunkBlocks) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
    blocks[0]
}

fn get_my_layer(blocks: &ChunkBlocks) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
    array::from_fn(|i| blocks[i][0])
}

fn get_mz_layer(blocks: &ChunkBlocks) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
    array::from_fn(|x| array::from_fn(|y| blocks[x][y][0]))
}

fn get_px_layer(blocks: &ChunkBlocks) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
    blocks[CHUNK_DIM - 1]
}

fn get_py_layer(blocks: &ChunkBlocks) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
    array::from_fn(|i| blocks[i][CHUNK_DIM - 1])
}

fn get_pz_layer(blocks: &ChunkBlocks) -> [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] {
    array::from_fn(|x| array::from_fn(|y| blocks[x][y][CHUNK_DIM - 1]))
}

pub fn get_adj_blocks(
    position: IVec3,
    chunks_map: &FxHashMap<IVec3, Chunk>,
) -> [[[VoxelBlock; CHUNK_DIM]; CHUNK_DIM]; 6] {
    let px = IVec3::new(position.x + 1, position.y, position.z);
    let py = IVec3::new(position.x, position.y + 1, position.z);
    let pz = IVec3::new(position.x, position.y, position.z + 1);

    let mx = IVec3::new(position.x - 1, position.y, position.z);
    let my = IVec3::new(position.x, position.y - 1, position.z);
    let mz = IVec3::new(position.x, position.y, position.z - 1);

    // fixme just pass none?
    [
        chunks_map
            .get(&px)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_mx_layer(&c.blocks)),
        chunks_map
            .get(&py)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_my_layer(&c.blocks)),
        chunks_map
            .get(&pz)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_mz_layer(&c.blocks)),
        chunks_map
            .get(&mx)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_px_layer(&c.blocks)),
        chunks_map
            .get(&my)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_py_layer(&c.blocks)),
        chunks_map
            .get(&mz)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_pz_layer(&c.blocks)),
    ]
}
