const CHUNK_DIM_U16 = 16u;
const CHUNK_DIM_U32 = 8u;
const TILE_DIM: f32 = 0.5;


@group(0) @binding(0)
var<storage, read_write> chunk_storage: array<GPUChunkEntry>;
@group(0) @binding(1)
var<storage, read_write> vertex_buffer: VertexBuffer;
@group(0) @binding(2)
var<storage, read_write> index_buffer: IndexBuffer;
@group(0) @binding(3)
var<storage, read_write> chunk_model_mats_buffer: array<mat4x4<f32>>;


var<private> chunk_vertex_offset: u32 = 0u;
var<private> chunk_index_offset: u32 = 0u;


@compute @workgroup_size(1)
fn compute_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let chunk_index = global_id.x;
    let chunk = chunk_storage[chunk_index];
//    chunk_vertex_offset = chunk.vertex_offset;
//    chunk_index_offset = chunk.index_offset;
    let temp_uv: vec2<f32> = vec2<f32>(0.0, 0.0);
    let blocks = chunk.blocks;
    calc_face_data(blocks);

    chunk_model_mats_buffer[chunk.slab_index] = model_matrix_from_position(chunk.world_position);
}
