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
var<workgroup> wg_chunk_content: GPUVoxelChunkContentWithAdj;
var<workgroup> wg_chunk_position: vec3<i32>;
var<workgroup> wg_chunk_index: u32;

var<private> pr_face_data: array<array<GPUVoxelFaceData, CHUNK_DIM>, 6>;
var<private> pr_face_counts: array<u32, 6> = array<u32, 6>(0u, 0u, 0u, 0u, 0u, 0u);

@compute @workgroup_size(CHUNK_DIM, CHUNK_DIM, 1)
fn mesh_chunks_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    // initialize workgroup
    if (lid.x + lid.y == 0u) {
        let mesh_entry = mesh_queue_buffer[wid.x];
        let header = chunks_meta_buffer[mesh_entry.index];
        let face_counts: array<u32, 6> = mesh_entry_face_counts(header.faces_positive, header.faces_negative);
        let face_offsets: array<u32, 6> = mesh_entry_face_offsets(mesh_entry.face_alloc, face_counts);
        atomicStore(&wg_face_buffer_write_offsets[0], face_offsets[0]);
        atomicStore(&wg_face_buffer_write_offsets[1], face_offsets[1]);
        atomicStore(&wg_face_buffer_write_offsets[2], face_offsets[2]);
        atomicStore(&wg_face_buffer_write_offsets[3], face_offsets[3]);
        atomicStore(&wg_face_buffer_write_offsets[4], face_offsets[4]);
        atomicStore(&wg_face_buffer_write_offsets[5], face_offsets[5]);

        wg_chunk_index = header.index;
        wg_chunk_position = header.position;
    }
    workgroupBarrier();

    // initialize 3d arr with adj blocks included
    let x = lid.x;
    let y = lid.y;
    let offs_x = x + 1u;
    let offs_y = y + 1u;

    wg_chunk_content.blocks[0][offs_x][offs_y] = chunks_data_b_buffer[wg_chunk_index].prev_blocks[0][x][y];
    wg_chunk_content.blocks[offs_x][0][offs_y] = chunks_data_b_buffer[wg_chunk_index].prev_blocks[1][x][y];
    wg_chunk_content.blocks[CHUNK_DIM + 1u][offs_x][offs_y] = chunks_data_b_buffer[wg_chunk_index].next_blocks[0][x][y];
    wg_chunk_content.blocks[offs_x][CHUNK_DIM + 1u][offs_y] = chunks_data_b_buffer[wg_chunk_index].next_blocks[1][x][y];
    wg_chunk_content.blocks[offs_x][offs_y][0] = chunks_data_b_buffer[wg_chunk_index].prev_blocks[2][x][y];
    wg_chunk_content.blocks[offs_x][offs_y][CHUNK_DIM + 1u] = chunks_data_b_buffer[wg_chunk_index].next_blocks[2][x][y];

    for (var z = 0u; z < CHUNK_DIM; z++) {
        wg_chunk_content.blocks[offs_x][offs_y][z + 1u] = chunks_data_a_buffer[wg_chunk_index].blocks[x][y][z];
    }
    workgroupBarrier();

    meshing_pass(lid.x, lid.y);
}
