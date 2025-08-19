
@group(0) @binding(0)
var<storage, read_write> chunk_entries: ChunkEntryBuffer;
@group(0) @binding(1)
var<storage, read_write> staging_vertex_buffer: VertexBuffer;
@group(0) @binding(2)
var<storage, read_write> staging_index_buffer: IndexBuffer;
@group(0) @binding(3)
var<storage, read_write> staging_mmat_buffer: array<mat4x4<f32>>;

var<workgroup> vertex_count: atomic<u32>;
var<workgroup> index_count: atomic<u32>;

@compute @workgroup_size(CHUNK_DIM, CHUNK_DIM, 1)
fn compute_main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let chunk_index = global_id.x;

    if (chunk_index < chunk_entries.count) {
        let chunk = chunk_entries.chunks[chunk_index];
        let chunk_header = chunk.header;
        var blocks = chunk.blocks;
        atomicStore(&vertex_count, chunk_header.vertex_offset);
        atomicStore(&index_count, chunk_header.index_offset);

        mesh_chunk_position(
            &blocks,
            local_id.x,
            local_id.y,
        );

        let chunk_world_position = chunk_to_world_position(chunk_header.chunk_position);
        staging_mmat_buffer[chunk_header.slab_index] = translation_matrix(chunk_world_position);
    }
}
