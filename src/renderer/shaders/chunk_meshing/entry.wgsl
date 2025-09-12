
@group(0) @binding(0)
var<storage, read> chunk_entries_buffer: ChunkEntryBuffer;
@group(0) @binding(1)
var<storage, read_write> face_data_buffer: array<FaceData>;
@group(0) @binding(2)
var<storage, read_write> mmat_buffer: array<vec3<f32>>;

var<workgroup> workgroup_buffer_write_offset: atomic<u32>;
var<workgroup> workgroup_chunk_blocks: ChunkBlocks;
var<workgroup> workgroup_chunk_adj_blocks: ChunkAdjacentBlocks;

var<private> private_face_data: array<FaceData, MAX_FACES_PER_THREAD>;
var<private> private_face_count: u32 = 0u;

@compute @workgroup_size(CHUNK_DIM, CHUNK_DIM, 1)
fn mesh_chunks_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let chunk_index = wid.x;
    let chunk_header = chunk_entries_buffer[chunk_index].header;

    if (lid.x + lid.y == 0u) {
        // first thread initializes workgroup vars
        workgroup_chunk_blocks = chunk_entries_buffer[chunk_index].blocks;
        workgroup_chunk_adj_blocks = chunk_entries_buffer[chunk_index].adjacent_blocks;
        atomicStore(&workgroup_buffer_write_offset, chunk_header.buffer_data.offset);
    }
    workgroupBarrier();

    mesh_chunk_position(lid.x, lid.y);

    let chunk_world_position = chunk_to_world_position(chunk_header.position);
    mmat_buffer[chunk_header.slab_index] = chunk_world_position;
}
