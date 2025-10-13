
@group(0) @binding(0)
var<storage, read_write> chunks_buffer_dst: array<GPUVoxelChunk>;
@group(0) @binding(1)
var<storage, read> chunks_buffer_src: array<GPUVoxelChunk>;

var<workgroup> workgroup_write_count: u32;

const TEMP = MAX_WORKGROUP_DIM_2D * MAX_WORKGROUP_DIM_2D;

@compute @workgroup_size(TEMP, 1, 1)
fn chunk_write_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    if (lid.x == 0u) {
        // first thread initializes workgroup vars
        // slab_index of chunk 0 in src buffer is the write_count (chunks start at index 1)
        workgroup_write_count = chunks_buffer_src[0].header.slab_index;
    }
    workgroupBarrier();

    let write_mapping = vec2<u32>(lid.x, wid.x);
    add_new_chunk(write_mapping);
}

// fixme add write mapping math util as general shader code
fn src_index_from_mapping(write_mapping: vec2<u32>) -> u32 {
    let wg_offset = write_mapping.y * TEMP;
    let x_offset = write_mapping.x;
    return 1 + wg_offset + x_offset;
}

fn src_exists(src_index: u32) -> bool {
    return src_index <= workgroup_write_count;
}

fn add_new_chunk(write_mapping: vec2<u32>) {
    let src_index = src_index_from_mapping(write_mapping);
    if (src_exists(src_index)) {
        let chunk = chunks_buffer_src[src_index];
        let dst_index = chunk.header.slab_index;
        chunks_buffer_dst[dst_index] = chunk;
    }
}
