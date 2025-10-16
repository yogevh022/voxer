
@group(0) @binding(0)
var<storage, read_write> chunks_buffer_dst: array<GPUVoxelChunk>;
@group(0) @binding(1)
var<storage, read> chunks_buffer_src: array<GPUVoxelChunk>;

var<push_constant>  input_length: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn chunk_write_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let src_index = thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);
    if (src_index < input_length) {
        let chunk = chunks_buffer_src[src_index];
        let dst_index = chunk.header.index;
        chunks_buffer_dst[dst_index] = chunk;
    }
}
