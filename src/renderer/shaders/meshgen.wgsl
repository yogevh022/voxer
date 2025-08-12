const CHUNK_DIM_U16 = 16u;
const CHUNK_DIM_U32 = 8u;
const CHUNK_SLICE_U32 = 128u;
const TILE_DIM: f32 = 16.0;


struct GPUChunkEntry {
    exists: atomic<u32>,
    vertex_offset: u32,
    index_offset: u32,
    vertex_count: u32,
    index_count: u32,
    blocks: ChunkBlocks,
}

struct ChunkStorage {
    data: array<GPUChunkEntry>,
}

struct Vertex {
    position: vec4<f32>,
    tex_coords: vec2<f32>,
}

struct LayerFaceData {
    faces: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>,
    dirs: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>,
}

@group(0) @binding(0)
var<storage, read_write> chunk_storage: ChunkStorage;
@group(0) @binding(1)
var<storage, read_write> vertex_buffer: array<Vertex>;
@group(0) @binding(2)
var<storage, read_write> index_buffer: array<u32>;
@group(0) @binding(3)
var<storage, read_write> chunk_model_mats_buffer: array<mat4x4<f32>>;


var<private> chunk_face: array<array<LayerFaceData, CHUNK_DIM_U16>, 3>;
var<private> rot_output: ChunkBlocks;

fn bit_at(value: u32, index: u32) -> u32 {
    return (value >> index) & 1u;
}

fn rotate_z_bits(arr: ChunkBlocks) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            rot_output[y][CHUNK_DIM_U16 - 1 - x] = arr[x][y];
        }
    }
}

fn rotate_y_bits(arr: ChunkBlocks) {
    rot_output = array<array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>, CHUNK_DIM_U16>();
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z += 1u) {
                // rot_output[z][y] |= bit_at(arr[x][y], z) << (CHUNK_DIM_U32 - 1u - x);
                rot_output[z][y][z] = arr[x][y][z];
            }
        }
    }
}

fn calc_face_data(blocks: ChunkBlocks, face_axis: u32) {
    for (var i: u32 = 0u; i < CHUNK_DIM_U16 - 1u; i += 1u) { // x
        var arr_a: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16> = blocks[i];
        var arr_b: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16> = blocks[i+1u];
        for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {  // y
            for (var y: u32 = 0u; y < CHUNK_DIM_U32; y += 1u) { // z
                chunk_face[face_axis][i].faces[x][y] = arr_a[x][y] ^ arr_b[x][y];
                chunk_face[face_axis][i].dirs[x][y] = arr_a[x][y] & (~arr_b[x][y]);
            }
        }
    }
}

fn write_quad_indices(base_index: u32, offset: u32) {
    index_buffer[index_buff_offset + base_index + 0u] = offset + 0u;
    index_buffer[index_buff_offset + base_index + 1u] = offset + 1u;
    index_buffer[index_buff_offset + base_index + 2u] = offset + 2u;
    index_buffer[index_buff_offset + base_index + 3u] = offset + 0u;
    index_buffer[index_buff_offset + base_index + 4u] = offset + 2u;
    index_buffer[index_buff_offset + base_index + 5u] = offset + 3u;
}

fn mesh() {
    let temp_uv: vec2<f32> = vec2<f32>(0.5, 0.5);
    var vert_count: u32 = 0u;
    var ind_count: u32 = 0u;
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z += 1u) {
                if ((chunk_face[0u][x].faces[y][z] & (1 << 15)) != 0) {
                        write_quad_indices(ind_count, vert_count);
                    if ((chunk_face[0u][x].dirs[y][z] >> 15) == 0 ) {
                        //minus_x_vertices(vert_count, temp_uv, f32(x), f32(y), f32(z));
                    } else {
                        //plus_x_vertices(vert_count, temp_uv, f32(x), f32(y), f32(z));
                    }
                    vert_count += 4u;
                    ind_count += 6u;
                }
            }
        }
    }
}

fn tech() {
    var vert_count: u32 = 0u;
    var ind_count: u32 = 0u;
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {
        let axis_1 = chunk_face[0u][x];
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            let axis_1_faces_y = axis_1.faces[y];
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z += 1u) {
                if ((axis_1_faces_y[z] & (1 << 15)) != 0) {
                    write_quad_indices(ind_count, vert_count);
                }
            }
        }
    }
}

fn plus_x_vertices(
    base_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer[vertex_buff_offset + base_index + 0].position = vec4<f32>(x, y + 1.0, z, 1.0);
    vertex_buffer[vertex_buff_offset + base_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_buff_offset + base_index + 1].position = vec4<f32>(x, y, z, 1.0);
    vertex_buffer[vertex_buff_offset + base_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer[vertex_buff_offset + base_index + 2].position = vec4<f32>(x, y, z + 1.0, 1.0);
    vertex_buffer[vertex_buff_offset + base_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer[vertex_buff_offset + base_index + 3].position = vec4<f32>(x, y + 1.0, z + 1.0, 1.0);
    vertex_buffer[vertex_buff_offset + base_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

fn minus_x_vertices(
    base_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer[vertex_buff_offset + base_index + 0].position = vec4<f32>(x, y + 1.0, z + 1.0, 1.0);
    vertex_buffer[vertex_buff_offset + base_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_buff_offset + base_index + 1].position = vec4<f32>(x, y, z + 1.0, 1.0);
    vertex_buffer[vertex_buff_offset + base_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer[vertex_buff_offset + base_index + 2].position = vec4<f32>(x, y, z, 1.0);
    vertex_buffer[vertex_buff_offset + base_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer[vertex_buff_offset + base_index + 3].position = vec4<f32>(x, y + 1.0, z, 1.0);
    vertex_buffer[vertex_buff_offset + base_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

var<private> vertex_buff_offset: u32 = 0u;
var<private> index_buff_offset: u32 = 0u;

@compute @workgroup_size(256)
fn compute_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let block_index = global_id.x;
    let blocks = chunk_storage.data[block_index].blocks;

    calc_face_data(blocks, 0u);
    rotate_z_bits(blocks);
    calc_face_data(rot_output, 1u);
    rotate_y_bits(blocks);
    calc_face_data(rot_output, 2u);

    vertex_buff_offset = chunk_storage.data[block_index].vertex_offset;
    index_buff_offset = chunk_storage.data[block_index].index_offset;

//    mesh();
    tech();
}



alias ChunkBlocks = array<array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>, CHUNK_DIM_U16>; // wgsl has no u16 :D
