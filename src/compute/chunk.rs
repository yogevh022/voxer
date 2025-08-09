use crate::worldgen::types::{CHUNK_SIZE, Chunk};
use simdeez::Simd;
use simdeez::avx2::Avx2;
use simdeez::scalar::Scalar;
use simdeez::simd_runtime_generate;
use simdeez::sse41::Sse2;
use simdeez::sse41::Sse41;

type ChunkBits = [u16; CHUNK_SIZE * CHUNK_SIZE];
type ChunkLayerBits = [u16; CHUNK_SIZE];
pub const OPAQUE_LAYER: ChunkLayerBits = [1; CHUNK_SIZE];
pub struct ChunkLayerBitData {
    pub faces: ChunkLayerBits,
    pub directions: ChunkLayerBits,
}

pub struct ChunkAxisFaceData {
    pub faces: ChunkBits,
    pub directions: ChunkBits,
}

pub struct ChunkFaceData {
    pub x: ChunkAxisFaceData,
    pub y: ChunkAxisFaceData,
    pub z: ChunkAxisFaceData,
}

impl ChunkFaceData {
    #[inline]
    pub fn face_count(&self) -> usize {
        self.x
            .faces
            .iter()
            .map(|r| r.count_ones() as usize)
            .sum::<usize>()
            + self
                .y
                .faces
                .iter()
                .map(|r| r.count_ones() as usize)
                .sum::<usize>()
            + self
                .z
                .faces
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
) -> ChunkFaceData {
    let bits = block_bits(chunk);
    let x_data = first_axis_face_data(&bits, next_x);

    let y_axis_bits = rotate_forward_y(&bits);
    let y_data = first_axis_face_data(&y_axis_bits, next_y);
    let y_data = ChunkAxisFaceData {
        faces: rotate_backward_y(&y_data.faces),
        directions: rotate_backward_y(&y_data.directions),
    };

    let z_axis_bits = rotate_forward_z(&bits);
    let z_data = first_axis_face_data(&z_axis_bits, next_z);
    let z_data = ChunkAxisFaceData {
        faces: rotate_backward_z(&z_data.faces),
        directions: rotate_backward_z(&z_data.directions),
    };

    ChunkFaceData {
        x: x_data,
        y: y_data,
        z: z_data,
    }
}

fn first_axis_face_data(bits: &ChunkBits, next_layer: &ChunkLayerBits) -> ChunkAxisFaceData {
    let mut faces_2d: [[u16; CHUNK_SIZE]; CHUNK_SIZE] = [[0u16; CHUNK_SIZE]; CHUNK_SIZE];
    let mut dirs_2d: [[u16; CHUNK_SIZE]; CHUNK_SIZE] = [[0u16; CHUNK_SIZE]; CHUNK_SIZE];

    for i in 0..CHUNK_SIZE - 1 {
        let layer_a = chunk_bits_layer(&bits, i);
        let layer_b = chunk_bits_layer(&bits, i + 1);
        let face_data = layer_face_data_runtime_select(layer_a, layer_b);
        faces_2d[i] = face_data.faces;
        dirs_2d[i] = face_data.directions;
    }
    let layer_b = chunk_bits_layer(&bits, CHUNK_SIZE - 1);
    let face_data = layer_face_data_runtime_select(layer_b, next_layer);
    faces_2d[CHUNK_SIZE - 1] = face_data.faces;
    dirs_2d[CHUNK_SIZE - 1] = face_data.directions;

    unsafe {
        ChunkAxisFaceData {
            faces: std::mem::transmute(faces_2d),
            directions: std::mem::transmute(dirs_2d),
        }
    }
}

simd_runtime_generate!(
    fn layer_face_data(a_u16: &ChunkLayerBits, b_u16: &ChunkLayerBits) -> ChunkLayerBitData {
        let a_i32: &[i32; CHUNK_SIZE / 2] = unsafe { std::mem::transmute(a_u16) };
        let b_i32: &[i32; CHUNK_SIZE / 2] = unsafe { std::mem::transmute(b_u16) };

        let mut faces = [0i32; CHUNK_SIZE / 2];
        let mut directions = [0i32; CHUNK_SIZE / 2];
        let s_chunks = (CHUNK_SIZE / 2) / S::VF32_WIDTH;
        unsafe {
            for i in 0..s_chunks {
                let va = S::loadu_epi32(&a_i32[i * S::VF32_WIDTH]);
                let vb = S::loadu_epi32(&b_i32[i * S::VF32_WIDTH]);

                let face_bits = S::xor_epi32(va, vb);
                let not_vb = S::not_epi32(vb);
                let dir_bits = S::and_epi32(va, not_vb);

                S::storeu_epi32(&mut faces[i * S::VF32_WIDTH], face_bits);
                S::storeu_epi32(&mut directions[i * S::VF32_WIDTH], dir_bits);
            }
        }

        unsafe {
            ChunkLayerBitData {
                faces: std::mem::transmute(faces),
                directions: std::mem::transmute(directions),
            }
        }
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

fn rotate_forward_z(arr: &ChunkBits) -> ChunkBits {
    let mut output = [0u16; CHUNK_SIZE * CHUNK_SIZE];
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            output[y * CHUNK_SIZE + (CHUNK_SIZE - 1 - x)] = arr[x * CHUNK_SIZE + y];
        }
    }
    output
}

fn rotate_backward_z(arr: &ChunkBits) -> ChunkBits {
    let mut output = [0u16; CHUNK_SIZE * CHUNK_SIZE];
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            output[x * CHUNK_SIZE + y] = arr[y * CHUNK_SIZE + (CHUNK_SIZE - 1 - x)];       
        }
    }
    output
}

fn rotate_forward_y(arr: &ChunkBits) -> ChunkBits {
    let mut output = [0u16; CHUNK_SIZE * CHUNK_SIZE];
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let z_data = arr[(CHUNK_SIZE - 1 - z) * CHUNK_SIZE + y];
            for x in 0..CHUNK_SIZE {
                if (z_data >> (CHUNK_SIZE - 1 - x)) & 1 == 1 {
                    output[x * CHUNK_SIZE + y] |= 1 << z; // shift by z we determine new z here
                    // ^ into > z_data[max-x]
                }
            }
        }
    }
    output
}

fn rotate_backward_y(arr: &ChunkBits) -> ChunkBits {
    let mut output = [0u16; CHUNK_SIZE * CHUNK_SIZE];
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let z_data = arr[x * CHUNK_SIZE + y];
            for z in 0..CHUNK_SIZE {
                if (z_data >> z) & 1 == 1 {
                    output[(CHUNK_SIZE - 1 - z) * CHUNK_SIZE + y] |= 1 << (CHUNK_SIZE - 1 - x);
                }
            }
        }
    }
    output
}