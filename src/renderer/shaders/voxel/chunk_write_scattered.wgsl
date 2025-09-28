@group(0) @binding(0)
var<storage, read_write> chunks_buffer: array<GPUVoxelChunk>;
@group(0) @binding(2)
var<storage, read> chunks_staging_buffer: array<GPUVoxelChunk>;

var<workgroup> workgroup_staging_length: u32;

@compute @workgroup_size(WORKGROUP_SIZE, 1)
fn chunk_write_scattered_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let staging_idx: u32 = (wid.x * WORKGROUP_SIZE) + lid.x;
    if (wid.x + lid.x == 0u) {
        // first thread initializes workgroup vars
        workgroup_staging_length = u32(chunks_staging_buffer[0].header.position.x);
    }
    workgroupBarrier();

    if (staging_idx < workgroup_staging_length) {
        let chunk = chunks_staging_buffer[staging_idx + 1];
        chunks_buffer[chunk.header.slab_index] = chunk;
    }
}