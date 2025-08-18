
@group(0) @binding(0)
var<storage, read_write> chunk_entries: ChunkEntryBuffer;
@group(0) @binding(1)
var<storage, read_write> vertex_buffer: VertexBuffer;
@group(0) @binding(2)
var<storage, read_write> index_buffer: IndexBuffer;
@group(0) @binding(3)
var<storage, read_write> chunk_model_mats_buffer: array<mat4x4<f32>>;

@compute @workgroup_size(128)
fn compute_main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    let chunk_index = local_id.x;

    if (chunk_index < chunk_entries.count) {
        let chunk = chunk_entries.chunks[chunk_index];
        let chunk_header = chunk.header;

        mesh_chunk_to_buffers(
            chunk.blocks,
            chunk_header.vertex_offset,
            chunk_header.index_offset
        );
        
        let chunk_world_position = chunk_to_world_position(chunk_header.chunk_position);
        chunk_model_mats_buffer[chunk_header.slab_index] = translation_matrix(chunk_world_position);
    }
}
