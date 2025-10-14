
@group(0) @binding(0)
var<storage, read_write> chunks_buffer_dst: array<GPUVoxelChunk>;
@group(0) @binding(1)
var<storage, read> chunks_buffer_src: array<GPUVoxelChunk>;

var<workgroup> workgroup_write_count: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn chunk_write_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    if (lid.x == 0u) {
        // slab_index of chunk 0 in src buffer is the write_count (chunks start at index 1)
        workgroup_write_count = chunks_buffer_src[0].header.index;
    }
    workgroupBarrier();

    let src_index = 1 + thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);
    if (src_index <= workgroup_write_count) {
        let chunk = chunks_buffer_src[src_index];
        let dst_index = chunk.header.index;
        chunks_buffer_dst[dst_index] = chunk;
    }
}
