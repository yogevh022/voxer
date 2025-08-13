const CHUNK_DIM_U16 = 16u;
const CHUNK_DIM_U32 = 8u;
const VOID_REF_OFFSET = 128u;
const TILE_DIM: f32 = 16.0;


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
        chunk_model_mats_buffer[chunk.slab_index] = model_matrix_from_position(chunk.world_position);
    }
}
