use crate::compute;
use crate::compute::array::Array3D;
use crate::world::types::{Block, BlockBytewise, CHUNK_DIM, CHUNK_SLICE, ChunkBlocks};
use std::array;

pub const OPAQUE_BITS_SLICE: [u16; CHUNK_DIM] = [1u16 << 15; CHUNK_DIM];

pub fn face_count(blocks: &ChunkBlocks) -> usize {
    let packed_blocks = pack_solid_blocks(blocks);

    let faces = faces(packed_blocks);
    faces.iter().map(|b| b.count_ones() as usize).sum::<usize>()
}

pub fn faces(packed_blocks: [u16; CHUNK_SLICE]) -> [u16; CHUNK_SLICE * 3] {
    let mut result = [0u16; CHUNK_SLICE * 3];
    let result_layers: &mut [[u16; CHUNK_DIM]; CHUNK_DIM * 3] =
        unsafe { &mut *(result.as_mut_ptr() as *mut [[u16; CHUNK_DIM]; CHUNK_DIM * 3]) };

    for i in 0..CHUNK_DIM - 1 {
        // y faces
        let ya: [u16; CHUNK_DIM] = packed_blocks[i * CHUNK_DIM..(i + 1) * CHUNK_DIM]
            .try_into()
            .unwrap();
        let yb: [u16; CHUNK_DIM] = packed_blocks[(i + 1) * CHUNK_DIM..(i + 2) * CHUNK_DIM]
            .try_into()
            .unwrap();

        // z faces
        let mut slice_za_iterator = packed_blocks.iter().cloned().skip(i).step_by(CHUNK_DIM);
        let mut slice_zb_iterator = packed_blocks.iter().cloned().skip(i + 1).step_by(CHUNK_DIM);
        let za: [u16; CHUNK_DIM] = array::from_fn(|_| slice_za_iterator.next().unwrap());
        let zb: [u16; CHUNK_DIM] = array::from_fn(|_| slice_zb_iterator.next().unwrap());

        // x faces
        let xb: [u16; CHUNK_DIM] = array::from_fn(|i| ya[i] >> 1);

        result_layers[i * 3] = compute::array::xor(&ya, &yb);
        result_layers[(i * 3) + 1] = compute::array::xor(&za, &zb);
        result_layers[(i * 3) + 2] = compute::array::xor(&ya, &xb);
    }

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
