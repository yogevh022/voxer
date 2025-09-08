
@group(0) @binding(0)
var<storage, read> chunk_entries: ChunkEntryBuffer;
@group(0) @binding(1)
var<storage, read_write> vertex_buffer: VertexBuffer;
@group(0) @binding(2)
var<storage, read_write> index_buffer: IndexBuffer;
@group(0) @binding(3)
var<storage, read_write> mmat_buffer: array<mat4x4<f32>>;

var<workgroup> workgroup_write_offset: atomic<u32>;
var<workgroup> workgroup_chunk_blocks: ChunkBlocks;
var<workgroup> workgroup_chunk_adj_blocks: ChunkAdjacentBlocks;

var<private> local_vertex_array: array<Vertex, MAX_VERTICES_PER_THREAD>;
var<private> local_index_array: array<Index, MAX_INDICES_PER_THREAD>;
var<private> local_face_count: u32 = 0u;

@compute @workgroup_size(CHUNK_DIM, CHUNK_DIM, 1)
fn mesh_chunks_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let chunk_index = wid.x;
    let chunk_header = chunk_entries[chunk_index].header;

    if (lid.x + lid.y == 0u) {
        // first thread initializes workgroup vars
        workgroup_chunk_blocks = chunk_entries[chunk_index].blocks;
        workgroup_chunk_adj_blocks = chunk_entries[chunk_index].adjacent_blocks;
        atomicStore(&workgroup_write_offset, chunk_header.offset);
    }
    workgroupBarrier();

    mesh_chunk_position(lid.x, lid.y);

    let chunk_world_position = chunk_to_world_position(chunk_header.chunk_position);
    mmat_buffer[chunk_header.slab_index] = translation_matrix(chunk_world_position);
}
