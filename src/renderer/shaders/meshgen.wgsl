const CHUNK_DIM_U16 = 16u;
const CHUNK_DIM_U32 = 8u;
const VOID_REF_OFFSET = 128u;
const TILE_DIM: f32 = 16.0;

struct GPUChunkEntry {
    vertex_offset: u32,
    index_offset: u32,
    vertex_count: u32,
    index_count: u32,
    world_position: vec3<f32>,
    blocks: ChunkBlocks,
}


struct Vertex {
    position: vec3<f32>,
    tex_coords: vec2<f32>,
}

struct LayerFaceData {
    faces: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>,
    dirs: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>,
}

struct IndexBuffer {
    indices: array<u32>,
}

struct VertexBuffer {
    vertices: array<Vertex>,
}

@group(0) @binding(0)
var<storage, read_write> chunk_storage: array<GPUChunkEntry>;
@group(0) @binding(1)
var<storage, read_write> vertex_buffer: VertexBuffer;
@group(0) @binding(2)
var<storage, read_write> index_buffer: IndexBuffer;
@group(0) @binding(3)
var<storage, read_write> chunk_model_mats_buffer: array<mat4x4<f32>>;


var<private> chunk_face: array<array<LayerFaceData, CHUNK_DIM_U16>, 3>;
var<private> rot_output: ChunkBlocks;

fn bit_at(value: u32, index: u32) -> u32 {
    return (value >> index) & 1u;
}

fn rotate_z(arr: ChunkBlocks) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            rot_output[y][CHUNK_DIM_U16 - 1 - x] = arr[x][y];
        }
    }
}

fn rotate_y(arr: ChunkBlocks) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x += 1u) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z += 1u) {
                rot_output[x][y][z] = arr[x][y][z];
            }
        }
    }
}

fn calc_face_data(blocks: ChunkBlocks, face_axis: u32) {
    for (var x: u32 = 0u; x < CHUNK_DIM_U16 - 1u; x += 1u) {
        var arr_a: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16> = blocks[x];
        var arr_b: array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16> = blocks[x+1u];
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y += 1u) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z += 1u) {
                chunk_face[face_axis][x].faces[y][z] = arr_a[y][z] ^ arr_b[y][z];
                chunk_face[face_axis][x].dirs[y][z] = arr_a[y][z] & (~arr_b[y][z]);
            }
        }
    }
}

fn quad_indices(index_index: u32, offset: u32) {
    index_buffer.indices[index_index + 0u] = offset + 0u;
    index_buffer.indices[index_index + 1u] = offset + 1u;
    index_buffer.indices[index_index + 2u] = offset + 2u;
    index_buffer.indices[index_index + 3u] = offset + 0u;
    index_buffer.indices[index_index + 4u] = offset + 2u;
    index_buffer.indices[index_index + 5u] = offset + 3u;
}

fn plus_x_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer.vertices[vertex_index + 0].position = vec3<f32>(x, y + 1.0, z);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer.vertices[vertex_index + 1].position = vec3<f32>(x, y, z);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 2].position = vec3<f32>(x, y, z + 1.0);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 3].position = vec3<f32>(x, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

fn minus_x_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer.vertices[vertex_index + 0].position = vec3<f32>(x, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer.vertices[vertex_index + 1].position = vec3<f32>(x, y, z + 1.0);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 2].position = vec3<f32>(x, y, z);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 3].position = vec3<f32>(x, y + 1.0, z);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

fn plus_y_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer.vertices[vertex_index + 0].position = vec3<f32>(x, y + 1.0, z);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer.vertices[vertex_index + 1].position = vec3<f32>(x, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 2].position = vec3<f32>(x + 1.0, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 3].position = vec3<f32>(x, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

fn minus_y_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer.vertices[vertex_index + 0].position = vec3<f32>(x, y, z + 1.0);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer.vertices[vertex_index + 1].position = vec3<f32>(x, y, z);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 2].position = vec3<f32>(x + 1.0, y, z);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 3].position = vec3<f32>(x + 1.0, y, z + 1.0);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

struct FaceMask {
    face_bit: u32,
    dir_bit: u32,
}

fn face_mask_for_axis(axis: u32, x: u32, y: u32, z: u32) -> FaceMask {
//        let face_bit = (chunk_face[0u][x].faces[y][z] >> ((16u << n) - 1u)) & 1u;
//        let dir_bit = (chunk_face[0u][x].dirs[y][z] >> ((16u << n) - 1u)) & 1u;
        return FaceMask(1u, 1u);
}

fn write_faces_x(
    index_offset: u32,
    index_count: ptr<function, u32>,
    vertex_offset: u32,
    vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.5, 0.5);

    let x_f32 = f32(x);
    let y_f32 = f32(y);
    let z_f32 = f32(z);

    // logic for both u16s packed into the u32
    for (var n = 0u; n < 2u; n += 1u) {
        let face_mask = face_mask_for_axis(0u, x, y, z);
        let i_masked = (VOID_REF_OFFSET + index_offset + (*index_count)) * face_mask.face_bit;
        let v_masked = (VOID_REF_OFFSET + vertex_offset + (*vertex_count)) * face_mask.face_bit;

        quad_indices(i_masked, v_masked);
        plus_x_vertices(v_masked * face_mask.dir_bit, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_x_vertices(v_masked * (1u ^ face_mask.dir_bit), temp_uv, x_f32, y_f32, z_f32);
        (*index_count) += (6u * face_mask.face_bit);
        (*vertex_count) += (4u * face_mask.face_bit);
    }
}

fn write_faces_y(
    index_offset: u32,
    index_count: ptr<function, u32>,
    vertex_offset: u32,
    vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.5, 0.5);

    let x_f32 = f32(x);
    let y_f32 = f32(y);
    let z_f32 = f32(z);

    // logic for both u16s packed into the u32
    for (var n = 0u; n < 2u; n += 1u) {
        let face_mask = face_mask_for_axis(1u, x, y, z);
        let i_masked = (VOID_REF_OFFSET + index_offset + (*index_count)) * face_mask.face_bit;
        let v_masked = (VOID_REF_OFFSET + vertex_offset + (*vertex_count)) * face_mask.face_bit;

        quad_indices(i_masked, v_masked);
        plus_y_vertices(v_masked * face_mask.dir_bit, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_y_vertices(v_masked * (1u ^ face_mask.dir_bit), temp_uv, x_f32, y_f32, z_f32);
        (*index_count) += (6u * face_mask.face_bit);
        (*vertex_count) += (4u * face_mask.face_bit);
    }
}

fn write_mesh_into_buffers(index_offset: u32, vertex_offset: u32) {
    var index_count: u32 = 0u;
    var vertex_count: u32 = 0u;
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x++) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y++) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z++) {
                write_faces_x(
                    index_offset,
                    &index_count,
                    vertex_offset,
                    &vertex_count,
                    x,
                    y,
                    z,
                );
                write_faces_y(
                    index_offset,
                    &index_count,
                    vertex_offset,
                    &vertex_count,
                    x,
                    y,
                    z,
                );
            }
        }
    }
}

fn model_matrix_from_position(position: vec3<f32>) -> mat4x4<f32> {
    var result: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(position, 1.0),
    );
    return result;
}

@compute @workgroup_size(256)
fn compute_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let chunk_index = global_id.x;
    let chunk = chunk_storage[chunk_index];
    if (chunk.vertex_offset != 0) {
        let blocks = chunk.blocks;
        calc_face_data(blocks, 0u);
        rotate_z(blocks);
        calc_face_data(rot_output, 1u);
        rotate_y(blocks);
        calc_face_data(rot_output, 2u);

        let index_offset = chunk.index_offset;
        let vertex_offset = chunk.vertex_offset;

        write_mesh_into_buffers(index_offset, vertex_offset);
        chunk_model_mats_buffer[chunk_index] = model_matrix_from_position(chunk.world_position);
    }
}



alias ChunkBlocks = array<array<array<u32, CHUNK_DIM_U32>, CHUNK_DIM_U16>, CHUNK_DIM_U16>; // wgsl has no u16 :D
