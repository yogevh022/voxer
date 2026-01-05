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
var<workgroup> wg_chunk_content: GPUVoxelChunkContentWithAdj;
var<workgroup> wg_chunk_position: vec3<i32>;
var<workgroup> wg_chunk_index: u32;

var<private> pr_face_data: array<array<GPUVoxelFaceData, MAX_DIR_FACES_PER_THREAD>, 6>;
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
    let half_y = lid.y / 2;
    let y_bit_pos = lid.y & 1u;
    let offs_x = lid.x + 1;
    let offs_y = lid.y + 1;
    let half_offs_y = (lid.y + 2) / 2;

    if (y_bit_pos == 0u) {
        wg_chunk_content.blocks[0][offs_x][half_offs_y] = chunks_data_b_buffer[wg_chunk_index].prev_blocks[0u][lid.x][half_y];
        wg_chunk_content.blocks[offs_x][0][half_offs_y] = chunks_data_b_buffer[wg_chunk_index].prev_blocks[1u][lid.x][half_y];
        wg_chunk_content.blocks[CHUNK_DIM + 1][offs_x][half_offs_y] = chunks_data_b_buffer[wg_chunk_index].next_blocks[0u][lid.x][half_y];
        wg_chunk_content.blocks[offs_x][CHUNK_DIM + 1][half_offs_y] = chunks_data_b_buffer[wg_chunk_index].next_blocks[1u][lid.x][half_y];
    }
    wg_chunk_content.blocks[offs_x][offs_y][0] = get_u16(chunks_data_b_buffer[wg_chunk_index].prev_blocks[2u][lid.x][half_y], y_bit_pos) >> 16;
    wg_chunk_content.blocks[offs_x][offs_y][CHUNK_DIM_HALF + 1] = get_u16(chunks_data_b_buffer[wg_chunk_index].next_blocks[2u][lid.x][half_y], y_bit_pos);

    for (var half_z = 0u; half_z < CHUNK_DIM_HALF; half_z++) {
        wg_chunk_content.blocks[offs_x][offs_y][half_z + 1] = chunks_data_a_buffer[wg_chunk_index].blocks[lid.x][lid.y][half_z];
    }
    workgroupBarrier();

    meshing_pass_at(lid.x, lid.y);
}
