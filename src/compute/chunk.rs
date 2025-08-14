use crate::compute;
use crate::compute::array::Array3D;
use crate::world::types::{Block, BlockBytewise, CHUNK_DIM, CHUNK_SLICE, ChunkBlocks};

pub const OPAQUE_BITS_SLICE: [u16; CHUNK_DIM] = [1u16 << 15; CHUNK_DIM];

pub fn face_count(blocks: &ChunkBlocks) -> usize {
    let packed_blocks = pack_solid_blocks(blocks);

    let packed_blocks_2d =
        unsafe { &*(packed_blocks.as_ptr() as *const [[u16; CHUNK_DIM]; CHUNK_DIM]) };
    // rot on z, y is on x
    let packed_rot_z = compute::array::rotated_z_bits(packed_blocks_2d);
    let y_blocks = unsafe { &*(packed_rot_z.as_ptr() as *const [u16; CHUNK_SLICE]) };
    // rot on y, z is on x
    let packed_rot_y =
        compute::array::rotated_y_bits::<CHUNK_DIM, CHUNK_DIM, CHUNK_DIM>(packed_blocks_2d);
    let z_blocks = unsafe { &*(packed_rot_y.as_ptr() as *const [u16; CHUNK_SLICE]) };

    let x_faces = faces_on_x(&packed_blocks, &OPAQUE_BITS_SLICE); // fixme chunk culling
    let y_faces = faces_on_x(y_blocks, &OPAQUE_BITS_SLICE);
    let z_faces = faces_on_x(z_blocks, &OPAQUE_BITS_SLICE);

    x_faces
        .iter()
        .map(|b| b.count_ones() as usize)
        .sum::<usize>()
        + y_faces
            .iter()
            .map(|b| b.count_ones() as usize)
            .sum::<usize>()
        + z_faces
            .iter()
            .map(|b| b.count_ones() as usize)
            .sum::<usize>()
}

fn faces_on_x(
    packed_blocks: &[u16; CHUNK_SLICE],
    next_slice: &[u16; CHUNK_DIM],
) -> [u16; CHUNK_SLICE] {
    let mut result = [0u16; CHUNK_SLICE];
    let result_layers: &mut [[u16; CHUNK_DIM]; CHUNK_DIM] =
        unsafe { &mut *(result.as_mut_ptr() as *mut [[u16; CHUNK_DIM]; CHUNK_DIM]) };

    for i in 0..CHUNK_DIM - 1 {
        let a = &packed_blocks[i * CHUNK_DIM..(i + 1) * CHUNK_DIM];
        let b = &packed_blocks[(i + 1) * CHUNK_DIM..(i + 2) * CHUNK_DIM];
        result_layers[i] = compute::array::xor(a.try_into().unwrap(), b.try_into().unwrap());
    }
    let a = &packed_blocks[CHUNK_DIM * CHUNK_DIM - CHUNK_DIM..];
    result_layers[CHUNK_DIM - 1] = compute::array::xor(a.try_into().unwrap(), next_slice);

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
