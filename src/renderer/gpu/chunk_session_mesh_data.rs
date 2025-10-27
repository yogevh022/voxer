use crate::compute::array::{Array3D, array_and, array_not, array_pop_count_u16, array_xor};
use crate::compute::bytes::bit_at;
use glam::U16Vec3;
use std::array;
use crate::world::{VoxelChunkAdjBlocks, VoxelChunkBlocks, CHUNK_DIM};
use crate::world::block::VoxelBlock;

pub const TRANSPARENT_LAYER_BLOCKS: [[VoxelBlock; CHUNK_DIM]; CHUNK_DIM] =
    [[VoxelBlock { value: 0 }; CHUNK_DIM]; CHUNK_DIM];

#[derive(Debug, Default, Clone, Copy)]
pub struct VoxelChunkMeshMeta {
    pub positive_faces: U16Vec3,
    pub negative_faces: U16Vec3,
}

pub(crate) fn chunk_mesh_data(
    blocks: &VoxelChunkBlocks,
    adj_blocks: &VoxelChunkAdjBlocks,
) -> VoxelChunkMeshMeta {
    type ChunkPositiveAdjBlocks = [[[VoxelBlock; CHUNK_DIM]; CHUNK_DIM]; 3];
    let positive_adj_blocks = unsafe { *adj_blocks.as_ptr().cast::<ChunkPositiveAdjBlocks>() };
    let packed_blocks = pack_solid_blocks(blocks);
    let packed_adj_blocks = pack_solid_blocks(&positive_adj_blocks);

    face_count_from_packed(packed_blocks, packed_adj_blocks)
}

fn face_count_from_packed(
    packed_blocks: [u16; CHUNK_DIM * CHUNK_DIM],
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
        mesh_meta.negative_faces.x += x_face_counts.0 as u16;
        mesh_meta.positive_faces.x += x_face_counts.1 as u16;

        let y_face_counts = bi_direction_face_counts(ya, yb);
        mesh_meta.negative_faces.y += y_face_counts.0 as u16;
        mesh_meta.positive_faces.y += y_face_counts.1 as u16;

        let z_face_counts = bi_direction_face_counts(xa, zb);
        mesh_meta.negative_faces.z += z_face_counts.0 as u16;
        mesh_meta.positive_faces.z += z_face_counts.1 as u16;
    }
    prep_adj_y(&packed_blocks, packed_adj_blocks[LAST_Y], ya, yb, LAST_X);
    prep_adj_z(packed_adj_blocks[LAST_Z], xb, zb);
    let adj_x = into_array_slice(&packed_adj_blocks[0..CHUNK_DIM]);

    let x_face_counts = bi_direction_face_counts(xb, adj_x);
    mesh_meta.negative_faces.x += x_face_counts.0 as u16;
    mesh_meta.positive_faces.x += x_face_counts.1 as u16;

    let y_face_counts = bi_direction_face_counts(ya, yb);
    mesh_meta.negative_faces.y += y_face_counts.0 as u16;
    mesh_meta.positive_faces.y += y_face_counts.1 as u16;

    let z_face_counts = bi_direction_face_counts(xb, zb);
    mesh_meta.negative_faces.z += z_face_counts.0 as u16;
    mesh_meta.positive_faces.z += z_face_counts.1 as u16;

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

fn into_array_slice<T, const N: usize>(slice: &[T]) -> &[T; N] {
    slice.try_into().unwrap()
}

#[inline(always)]
fn prep_adj_x(
    packed_blocks: &[u16; CHUNK_DIM * CHUNK_DIM],
    xa: &mut [u16; CHUNK_DIM],
    xb: &mut [u16; CHUNK_DIM],
    x: usize,
) {
    *xa = *into_array_slice(&packed_blocks[x * CHUNK_DIM..(x + 1) * CHUNK_DIM]);
    *xb = *into_array_slice(&packed_blocks[(x + 1) * CHUNK_DIM..(x + 2) * CHUNK_DIM]);
}

#[inline(always)]
fn prep_adj_y(
    packed_blocks: &[u16; CHUNK_DIM * CHUNK_DIM],
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
    blocks: &[[[VoxelBlock; Z]; Y]; X],
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
