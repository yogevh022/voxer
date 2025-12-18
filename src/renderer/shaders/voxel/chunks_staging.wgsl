
@group(0) @binding(0)
var<storage, read> chunks_staging_buffer: array<GPUVoxelChunk>;
@group(0) @binding(1)
var<storage, read_write> chunks_data_a_buffer: array<GPUVoxelChunkContent>;
@group(0) @binding(2)
var<storage, read_write> chunks_data_b_buffer: array<GPUVoxelChunkAdjContent>;
@group(0) @binding(3)
var<storage, read_write> chunks_meta_buffer: array<GPUVoxelChunkHeader>;

var<push_constant>  input_length: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn chunks_staging_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let src_index = thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);
    if (src_index < input_length) {
        let chunk = chunks_staging_buffer[src_index];
        let dst_index = chunk.header.index;
        chunks_data_a_buffer[dst_index] = chunk.content;
        chunks_data_b_buffer[dst_index] = chunk.adj_content;
        chunks_meta_buffer[dst_index] = chunk.header;
    }
}
