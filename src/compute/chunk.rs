use crate::worldgen::types::{CHUNK_SIZE, Chunk};
use simdeez::Simd;
use simdeez::avx2::Avx2;
use simdeez::scalar::Scalar;
use simdeez::simd_runtime_generate;
use simdeez::sse41::Sse2;
use simdeez::sse41::Sse41;

type ChunkBits = [u16; CHUNK_SIZE * CHUNK_SIZE];
type ChunkLayerBits = [u16; CHUNK_SIZE];
pub const OPAQUE_LAYER: ChunkLayerBits = [1u16; CHUNK_SIZE];
pub const TRANSPARENT_LAYER: ChunkLayerBits = [0u16; CHUNK_SIZE];

pub struct ChunkFaces {
    pub x: ChunkBits,
    pub y: ChunkBits,
    pub z: ChunkBits,
}

impl ChunkFaces {
    #[inline]
    pub fn face_count(&self) -> usize {
        self.x
            .iter()
            .map(|r| r.count_ones() as usize)
            .sum::<usize>()
            + self
                .y
                .iter()
                .map(|r| r.count_ones() as usize)
                .sum::<usize>()
            + self
                .z
                .iter()
                .map(|r| r.count_ones() as usize)
                .sum::<usize>()
    }
}

pub fn chunk_faces(
    chunk: &Chunk,
    next_x: &ChunkLayerBits,
    next_y: &ChunkLayerBits,
    next_z: &ChunkLayerBits,
) -> ChunkFaces {
    let bits = block_bits(chunk);
    let xor_x = xor_layer(&bits, next_x);

    let y_bits = transpose_y_to_x(&bits);
    let xor_y = xor_layer(&y_bits, next_y);

    let z_bits = transpose_z_to_x(&bits);
    let xor_z = xor_layer(&z_bits, next_z);

    ChunkFaces {
        x: xor_x,
        y: xor_y,
        z: xor_z,
    }
}

fn xor_layer(bits: &ChunkBits, next_layer: &ChunkLayerBits) -> ChunkBits {
    let mut xor_arr: [[u16; CHUNK_SIZE]; CHUNK_SIZE] = [[0u16; CHUNK_SIZE]; CHUNK_SIZE];

    for i in 0..CHUNK_SIZE - 1 {
        let layer_a = chunk_bits_layer(&bits, i);
        let layer_b = chunk_bits_layer(&bits, i + 1);
        xor_arr[i] = xor_bit_slices_runtime_select(layer_a, layer_b);
    }
    let layer_b = chunk_bits_layer(&bits, CHUNK_SIZE - 1);
    xor_arr[CHUNK_SIZE - 1] = xor_bit_slices_runtime_select(layer_b, next_layer);

    unsafe { std::mem::transmute(xor_arr) }
}

simd_runtime_generate!(
    fn xor_bit_slices(a_u16: &ChunkLayerBits, b_u16: &ChunkLayerBits) -> ChunkLayerBits {
        let a_i32: &[i32; CHUNK_SIZE / 2] = unsafe { std::mem::transmute(a_u16) };
        let b_i32: &[i32; CHUNK_SIZE / 2] = unsafe { std::mem::transmute(b_u16) };

        let mut result = [0i32; CHUNK_SIZE / 2];
        let s_chunks = (CHUNK_SIZE / 2) / S::VF32_WIDTH;
        unsafe {
            for i in 0..s_chunks {
                let va = S::loadu_epi32(&a_i32[i * S::VF32_WIDTH]);
                let vb = S::loadu_epi32(&b_i32[i * S::VF32_WIDTH]);

                let vx = S::xor_epi32(va, vb);
                S::storeu_epi32(&mut result[i * S::VF32_WIDTH], vx);
            }
        }

        unsafe { std::mem::transmute(result) }
    }
);

#[inline(always)]
fn chunk_bits_layer(bits: &ChunkBits, index: usize) -> &ChunkLayerBits {
    let layer = &bits[index * CHUNK_SIZE..(index + 1) * CHUNK_SIZE];
    layer.try_into().unwrap()
}

fn block_bits(chunk: &Chunk) -> ChunkBits {
    let mut bytes = [0u16; CHUNK_SIZE * CHUNK_SIZE];

    for (byte_idx, row) in chunk.blocks.iter().flatten().enumerate() {
        let mut bits = 0u16;
        for (i, b) in row.iter().enumerate() {
            if !b.is_transparent() {
                bits |= 1 << i;
            }
        }

        bytes[byte_idx] = bits;
    }

    bytes
}

fn transpose_y_to_x(arr: &ChunkBits) -> ChunkBits {
    let mut output = [0u16; CHUNK_SIZE * CHUNK_SIZE];
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            output[x * CHUNK_SIZE + y] = arr[y * CHUNK_SIZE + x];
        }
    }
    output
}

fn transpose_z_to_x(arr: &ChunkBits) -> ChunkBits {
    let mut output = [0u16; CHUNK_SIZE * CHUNK_SIZE];
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let mut bit = 0u16;
            for z in 0..CHUNK_SIZE {
                bit |= arr[z * CHUNK_SIZE + y] << x;
            }
            output[x * CHUNK_SIZE + y] = bit;
        }
    }
    output
}
