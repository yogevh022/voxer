use crate::compute::array::array_xor;
use crate::compute::array::{Array3D, array_pop_count_u16, array_pop_count_u32};
use crate::compute::bytes::bit_at;
use crate::world::types::{
    BlockBytewise, CHUNK_DIM, CHUNK_SLICE, Chunk, ChunkAdjacentBlocks, ChunkBlocks, VoxelBlock,
};
use glam::IVec3;
use rustc_hash::FxHashMap;
use std::array;

pub const TRANSPARENT_LAYER_BITS: [u16; CHUNK_DIM] = [0u16; CHUNK_DIM];
pub const TRANSPARENT_LAYER_BLOCKS: [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] =
    [[VoxelBlock { value: 0 }; CHUNK_DIM]; CHUNK_DIM];

pub fn face_count(blocks: &ChunkBlocks, adj_blocks: &ChunkAdjacentBlocks) -> usize {
    let packed_blocks = pack_solid_blocks(blocks);
    let packed_adjacent_blocks = pack_solid_blocks(adj_blocks);

    let face_count = faces(packed_blocks, packed_adjacent_blocks);
    face_count as usize
    // faces.iter().map(|b| b.count_ones() as usize).sum::<usize>()
}

fn faces(packed: [u16; CHUNK_SLICE], packed_adj: [u16; CHUNK_DIM * 6]) -> u32 {
    let xa = &mut [0u16; CHUNK_DIM];
    let xb = &mut [0u16; CHUNK_DIM];
    let ya = &mut [0u16; CHUNK_DIM + 1];
    let yb = &mut [0u16; CHUNK_DIM + 1];
    let za = &mut [0u32; CHUNK_DIM];
    let zb = &mut [0u32; CHUNK_DIM];

    let mut result = 0u32;
    *xa = packed_adj[(CHUNK_DIM * 3)..(CHUNK_DIM * 4)]
        .try_into()
        .unwrap();
    *xb = packed[0..CHUNK_DIM].try_into().unwrap();
    result += array_pop_count_u16(array_xor(xa, xb));

    for i in 0..CHUNK_DIM - 1 {
        adj_x(&packed, xa, xb, i);
        adj_y(&packed, &packed_adj, ya, yb, i);
        adj_z(&packed_adj, xa, za, zb, i);

        let xx = array_pop_count_u16(array_xor(xa, xb));
        let yx = array_pop_count_u16(array_xor(ya, yb));
        let zx = array_pop_count_u32(array_xor(za, zb));

        result += array_pop_count_u16(array_xor(xa, xb));
        result += array_pop_count_u16(array_xor(ya, yb));
        result += array_pop_count_u32(array_xor(za, zb));
    }
    adj_y(&packed, &packed_adj, ya, yb, CHUNK_DIM - 1);
    adj_z(&packed_adj, xb, za, zb, CHUNK_DIM - 1);
    result += array_pop_count_u16(array_xor(xb, &packed_adj[0..CHUNK_DIM].try_into().unwrap()));
    result += array_pop_count_u16(array_xor(ya, yb));
    result += array_pop_count_u32(array_xor(za, zb));
    result
}

#[inline]
fn adj_x(
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

#[inline]
fn adj_y(
    packed: &[u16; CHUNK_SLICE],
    packed_adj: &[u16; CHUNK_DIM * 6],
    ya: &mut [u16; CHUNK_DIM + 1],
    yb: &mut [u16; CHUNK_DIM + 1],
    x: usize,
) {
    let packed_adj_plus = packed_adj[CHUNK_DIM + x];
    let packed_adj_minus = packed_adj[CHUNK_DIM * 4 + x];

    ya[0] = packed_adj_minus;
    yb[0] = packed[x * CHUNK_DIM];
    for j in 0..CHUNK_DIM - 1 {
        ya[j + 1] = packed[(x * CHUNK_DIM) + j];
        yb[j + 1] = packed[(x * CHUNK_DIM) + j + 1];
    }
    ya[CHUNK_DIM] = packed[(x * CHUNK_DIM) + CHUNK_DIM - 1];
    yb[CHUNK_DIM] = packed_adj_plus;
}

#[inline]
fn adj_z(
    packed_adj: &[u16; CHUNK_DIM * 6],
    xa: &[u16; CHUNK_DIM],
    za: &mut [u32; CHUNK_DIM],
    zb: &mut [u32; CHUNK_DIM],
    x: usize,
) {
    let adj_plus = packed_adj[CHUNK_DIM * 2 + x];
    let adj_minus = packed_adj[CHUNK_DIM * 5 + x];
    *za = array::from_fn(|i| {
        let adj_plus_bit = bit_at(adj_plus, i) as u32;
        let adj_minus_bit = bit_at(adj_minus, i) as u32;
        adj_plus_bit << 17 | (xa[i] as u32) << 1 | adj_minus_bit
    });
    *zb = array::from_fn(|i| za[i] >> 1);
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
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_mx_layer(&c.blocks)),
        chunks_map
            .get(&my)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_my_layer(&c.blocks)),
        chunks_map
            .get(&mz)
            .map_or(TRANSPARENT_LAYER_BLOCKS, |c| get_mz_layer(&c.blocks)),
    ]
}
