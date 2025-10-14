const FACE_DATA_VOID_OFFSET: u32 = 1u;
const MAX_FACES_PER_THREAD: u32 = (3u * CHUNK_DIM) + FACE_DATA_VOID_OFFSET;

@group(0) @binding(0)
var<storage, read> chunks_buffer: array<GPUVoxelChunk>;
@group(0) @binding(1)
var<storage, read_write> face_data_buffer: array<GPUVoxelFaceData>;
@group(0) @binding(2)
var<storage, read> mesh_queue_buffer: array<GPUChunkMeshEntry>;

var<workgroup> workgroup_buffer_write_offset: atomic<u32>;
var<workgroup> workgroup_chunk_content: GPUVoxelChunkContent;
var<workgroup> workgroup_chunk_adj_content: GPUVoxelChunkAdjContent;
var<workgroup> workgroup_chunk_world_position: vec3<i32>;

var<private> private_face_data: array<GPUVoxelFaceData, MAX_FACES_PER_THREAD>;
var<private> private_face_count: u32 = 0u;

@compute @workgroup_size(CHUNK_DIM, CHUNK_DIM, 1)
fn mesh_chunks_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    if (lid.x + lid.y == 0u) {
        let mesh_entry = mesh_queue_buffer[wid.x];

        workgroup_chunk_content = chunks_buffer[mesh_entry.index].content;
        workgroup_chunk_adj_content = chunks_buffer[mesh_entry.index].adj_content;
        let chunk_header = chunks_buffer[mesh_entry.index].header;
        let chunk_position = vec3<i32>(
            chunk_header.chunk_x,
            chunk_header.chunk_y,
            chunk_header.chunk_z,
        );
        workgroup_chunk_world_position = chunk_position * i32(CHUNK_DIM);

        atomicStore(&workgroup_buffer_write_offset, mesh_entry.face_alloc);
    }
    workgroupBarrier();

    meshing_pass_at(lid.x, lid.y);
}
