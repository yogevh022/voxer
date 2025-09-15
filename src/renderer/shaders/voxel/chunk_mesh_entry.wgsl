const FACE_DATA_VOID_OFFSET: u32 = 1u;
const MAX_FACES_PER_THREAD: u32 = (3u * VCHUNK_DIM) + FACE_DATA_VOID_OFFSET;

@group(0) @binding(0)
var<storage, read> chunk_entries_buffer: array<VoxelChunkEntry>;
@group(0) @binding(1)
var<storage, read_write> face_data_buffer: array<VoxelFaceData>;

var<workgroup> workgroup_buffer_write_offset: atomic<u32>;
var<workgroup> workgroup_chunk_blocks: VoxelChunkBlocks;
var<workgroup> workgroup_chunk_adj_blocks: VoxelChunkAdjacentBlocks;
var<workgroup> workgroup_chunk_position: vec3<i32>;

var<private> private_face_data: array<VoxelFaceData, MAX_FACES_PER_THREAD>;
var<private> private_face_count: u32 = 0u;

@compute @workgroup_size(VCHUNK_DIM, VCHUNK_DIM, 1)
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
        workgroup_chunk_position = chunk_header.position;

        atomicStore(&workgroup_buffer_write_offset, chunk_header.buffer_data.offset);
    }
    workgroupBarrier();

    mesh_chunk_position(lid.x, lid.y);
}
