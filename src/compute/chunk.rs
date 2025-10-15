use crate::compute::array::{Array3D, array_and, array_not, array_pop_count_u16, array_xor};
use crate::compute::bytes::bit_at;
use crate::world::types::{
    BlockBytewise, CHUNK_DIM, CHUNK_SLICE, Chunk, ChunkAdjBlocks, ChunkBlocks, VoxelBlock,
};
use glam::{IVec3, U16Vec3};
use rustc_hash::FxHashMap;
use std::array;

#[derive(Debug, Default, Clone, Copy)]
pub struct VoxelChunkMeshMeta {
    pub positive_face_count: U16Vec3,
    pub negative_face_count: U16Vec3,
}

pub fn face_count(blocks: &ChunkBlocks, adj_blocks: &ChunkAdjBlocks) -> VoxelChunkMeshMeta {
    type ChunkPositiveAdjBlocks = Array3D<VoxelBlock, 3, CHUNK_DIM, CHUNK_DIM>;
    let positive_adj_blocks = unsafe { *adj_blocks.as_ptr().cast::<ChunkPositiveAdjBlocks>() };
    let packed_blocks = pack_solid_blocks(blocks);
    let packed_adj_blocks = pack_solid_blocks(&positive_adj_blocks);

    face_count_from_packed(packed_blocks, packed_adj_blocks)
}

fn face_count_from_packed(
    packed_blocks: [u16; CHUNK_SLICE],
    packed_adj_blocks: [u16; CHUNK_DIM * 3],
) -> VoxelChunkMeshMeta {
    let xa = &mut [0u16; CHUNK_DIM];
    let xb = &mut [0u16; CHUNK_DIM];
    let ya = &mut [0u16; CHUNK_DIM];
    let yb = &mut [0u16; CHUNK_DIM];
    let zb = &mut [0u16; CHUNK_DIM];

    const Y_OFFSET: usize = CHUNK_DIM + CHUNK_DIM;
    const Z_OFFSET: usize = CHUNK_DIM + CHUNK_DIM + CHUNK_DIM;
    const LAST_X: usize = CHUNK_DIM - 1;
    const LAST_Y: usize = Y_OFFSET - 1;
    const LAST_Z: usize = Z_OFFSET - 1;

    let mut mesh_meta = VoxelChunkMeshMeta::default();

    for i in 0..CHUNK_DIM - 1 {
        prep_adj_x(&packed_blocks, xa, xb, i);
        prep_adj_y(&packed_blocks, packed_adj_blocks[CHUNK_DIM + i], ya, yb, i);
        prep_adj_z(packed_adj_blocks[Y_OFFSET + i], xa, zb);

        let x_face_counts = bi_direction_face_counts(xa, xb);
        mesh_meta.negative_face_count.x += x_face_counts.0 as u16;
        mesh_meta.positive_face_count.x += x_face_counts.1 as u16;

        let y_face_counts = bi_direction_face_counts(ya, yb);
        mesh_meta.negative_face_count.y += y_face_counts.0 as u16;
        mesh_meta.positive_face_count.y += y_face_counts.1 as u16;

        let z_face_counts = bi_direction_face_counts(xa, zb);
        mesh_meta.negative_face_count.z += z_face_counts.0 as u16;
        mesh_meta.positive_face_count.z += z_face_counts.1 as u16;
    }
    prep_adj_y(&packed_blocks, packed_adj_blocks[LAST_Y], ya, yb, LAST_X);
    prep_adj_z(packed_adj_blocks[LAST_Z], xb, zb);
    let adj_x = as_sized_slice(&packed_adj_blocks[0..CHUNK_DIM]);

    let x_face_counts = bi_direction_face_counts(xb, adj_x);
    mesh_meta.negative_face_count.x += x_face_counts.0 as u16;
    mesh_meta.positive_face_count.x += x_face_counts.1 as u16;

    let y_face_counts = bi_direction_face_counts(ya, yb);
    mesh_meta.negative_face_count.y += y_face_counts.0 as u16;
    mesh_meta.positive_face_count.y += y_face_counts.1 as u16;

    let z_face_counts = bi_direction_face_counts(xb, zb);
    mesh_meta.negative_face_count.z += z_face_counts.0 as u16;
    mesh_meta.positive_face_count.z += z_face_counts.1 as u16;

    mesh_meta
}

fn bi_direction_face_counts(a: &[u16; CHUNK_DIM], b: &[u16; CHUNK_DIM]) -> (u32, u32) {
    let faces = array_xor(a, b);
    let not_b = array_not(b);
    let dirs = array_and(a, &not_b);
    let not_dirs = array_not(&dirs);
    let positive_faces = array_and(&faces, &dirs);
    let negative_faces = array_and(&faces, &not_dirs);

    (
        array_pop_count_u16(&negative_faces),
        array_pop_count_u16(&positive_faces),
    )
}

fn as_sized_slice<T, const N: usize>(slice: &[T]) -> &[T; N] {
    unsafe {
        let src = slice;
        &*(src.as_ptr() as *const _)
    }
}

#[inline(always)]
fn prep_adj_x(
    packed_blocks: &[u16; CHUNK_SLICE],
    xa: &mut [u16; CHUNK_DIM],
    xb: &mut [u16; CHUNK_DIM],
    x: usize,
) {
    let xa_slice = &packed_blocks[x * CHUNK_DIM..(x + 1) * CHUNK_DIM];
    *xa = *as_sized_slice(xa_slice);
    let xb_slice = &packed_blocks[(x + 1) * CHUNK_DIM..(x + 2) * CHUNK_DIM];
    *xb = *as_sized_slice(xb_slice);
}

#[inline(always)]
fn prep_adj_y(
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
fn prep_adj_z(packed_adjacent_blocks: u16, xa: &[u16; CHUNK_DIM], zb: &mut [u16; CHUNK_DIM]) {
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

pub const TRANSPARENT_LAYER_BITS: [u16; CHUNK_DIM] = [0u16; CHUNK_DIM];
pub const TRANSPARENT_LAYER_BLOCKS: [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] =
    [[VoxelBlock { value: 0 }; CHUNK_DIM]; CHUNK_DIM];

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
