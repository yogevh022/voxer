
@group(0) @binding(0)
var<storage, read> view_candidates_write_buffer: array<GPUChunkMeshEntryWrite>;
@group(0) @binding(1)
var<storage, read_write> view_candidates_buffer: array<GPUChunkMeshEntry>;

var<push_constant> input_length: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn view_candidate_write_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let src_index = thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);
    if (src_index < input_length) {
        let mesh_entry_write = view_candidates_write_buffer[src_index];
        let dst_index = mesh_entry_write.index;
        view_candidates_buffer[dst_index] = mesh_entry_write.entry;
    }
}
