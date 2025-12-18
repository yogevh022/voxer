const MAX_DIR_FACES_PER_THREAD: u32 = CHUNK_DIM + VOID_OFFSET;

@group(0) @binding(0)
var<storage, read> chunks_data_a_buffer: array<GPUVoxelChunkContent>;
@group(0) @binding(1)
var<storage, read> chunks_data_b_buffer: array<GPUVoxelChunkAdjContent>;
@group(0) @binding(2)
var<storage, read> chunks_meta_buffer: array<GPUVoxelChunkHeader>;
@group(0) @binding(3)
var<storage, read_write> face_data_buffer: array<GPUVoxelFaceData>;
@group(0) @binding(4)
var<storage, read> mesh_queue_buffer: array<GPUChunkMeshEntry>;

var<workgroup> wg_face_buffer_write_offsets: array<atomic<u32>, 6>;
var<workgroup> wg_chunk_content: GPUVoxelChunkContent;
var<workgroup> wg_chunk_adj_content: GPUVoxelChunkAdjContent;
var<workgroup> wg_chunk_position: vec3<i32>;

var<private> pr_face_data: array<array<GPUVoxelFaceData, MAX_DIR_FACES_PER_THREAD>, 6>;
var<private> pr_face_counts: array<u32, 6> = array<u32, 6>(0u, 0u, 0u, 0u, 0u, 0u);

@compute @workgroup_size(CHUNK_DIM, CHUNK_DIM, 1)
fn mesh_chunks_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    if (lid.x + lid.y == 0u) {
        let mesh_entry = mesh_queue_buffer[wid.x];
        let header = chunks_meta_buffer[mesh_entry.index];
        let face_counts: array<u32, 6> = mesh_face_counts(header.faces_positive, header.faces_negative);
        let face_offsets: array<u32, 6> = mesh_entry_face_offsets(mesh_entry.face_alloc, face_counts);
        atomicStore(&wg_face_buffer_write_offsets[0], face_offsets[0]);
        atomicStore(&wg_face_buffer_write_offsets[1], face_offsets[1]);
        atomicStore(&wg_face_buffer_write_offsets[2], face_offsets[2]);
        atomicStore(&wg_face_buffer_write_offsets[3], face_offsets[3]);
        atomicStore(&wg_face_buffer_write_offsets[4], face_offsets[4]);
        atomicStore(&wg_face_buffer_write_offsets[5], face_offsets[5]);

        wg_chunk_content = chunks_data_a_buffer[mesh_entry.index];
        wg_chunk_adj_content = chunks_data_b_buffer[mesh_entry.index];
        wg_chunk_position = header.position;
    }
    workgroupBarrier();

    meshing_pass_at(lid.x, lid.y);
}
