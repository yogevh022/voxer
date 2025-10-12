
@group(0) @binding(0)
var<storage, read> chunks_buffer_src: array<GPUVoxelChunk>;
@group(0) @binding(1)
var<storage, read_write> chunks_buffer_dst: array<GPUVoxelChunk>;

var<workgroup> workgroup_write_count: u32;

@compute @workgroup_size(CHUNK_DIM, CHUNK_DIM, 1)
fn chunks_write_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    if (lid.x + lid.y == 0u) {
        // first thread initializes workgroup vars
        // slab_index of chunk 0 in src buffer is the write_count (chunks start at index 1)
        workgroup_write_count = chunks_buffer_src[0].header.slab_index;
    }
    workgroupBarrier();

    let write_mapping = vec3<u32>(lid.x, lid.y, wid.x);
    add_new_chunk(write_mapping);
}

fn src_index_from_mapping(write_mapping: vec3<u32>) -> u32 {
    let wg_offset = write_mapping.z * CHUNK_DIM * CHUNK_DIM;
    let y_offset = write_mapping.y * CHUNK_DIM;
    let x_offset = write_mapping.x;
    return 1 + wg_offset + y_offset + x_offset;
}

fn can_write(src_index: u32) -> bool {
    return src_index <= workgroup_write_count;
}

fn add_new_chunk(write_mapping: vec3<u32>) {
    let src_index = src_index_from_mapping(write_mapping);
    if (can_write(src_index)) {
        let chunk = chunks_buffer_src[src_index];
        let dst_index = chunk.header.slab_index;
        chunks_buffer_dst[dst_index] = chunk;
    }
}
