
@group(0) @binding(0)
var<storage, read_write> indirect_buffer: array<GPUDrawIndirectArgs>;
@group(0) @binding(1)
var<storage, read_write> indirect_count_buffer: array<atomic<u32>>;
@group(0) @binding(2)
var<storage, read> chunks_buffer: array<GPUVoxelChunk>;
@group(0) @binding(3)
var<storage, read> chunks_in_view_buffer: array<u32>;
@group(0) @binding(4)
var<uniform> camera_view: UniformCameraView;

const MAX_DRAW_ARGS_PER_WORKGROUP: u32 = MAX_WORKGROUP_DIM_2D * MAX_WORKGROUP_DIM_2D;
var<workgroup> workgroup_indirect_draw_args: array<GPUDrawIndirectArgs, MAX_DRAW_ARGS_PER_WORKGROUP>;
var<workgroup> workgroup_indirect_draw_args_count: atomic<u32>;
var<workgroup> workgroup_max_entries: u32;

@compute @workgroup_size(MAX_DRAW_ARGS_PER_WORKGROUP, 1, 1)
fn write_chunk_indirect_draw_entry(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let wg_offset = wid.x * MAX_DRAW_ARGS_PER_WORKGROUP;
    let x_offset = lid.x;
    let draw_arg_index = wg_offset + x_offset;

    if (draw_arg_index == 0) {
        workgroup_max_entries = chunks_in_view_buffer[0];
    }
    workgroupBarrier();

    let ch_idx = chunks_in_view_buffer[draw_arg_index + 1];
    let ch_header = chunks_buffer[ch_idx].header;

    let ch_vertex_count = ch_header.buffer_data.face_count * 6u;
    let ch_first_vertex = ch_header.buffer_data.offset * 6u;
    let ch_position = ch_header.position;
    let ch_packed_xz = pack_u16s(bitcast<u32>(ch_position.x), bitcast<u32>(ch_position.z));
    let ch_min = vec3<f32>(ch_position);
    let ch_max = ch_min * f32(CHUNK_DIM);

    if (draw_arg_index < workgroup_max_entries) && frustum_check_chunk(ch_min, ch_max) {
        let arg_idx = atomicAdd(&workgroup_indirect_draw_args_count, 1);
        let draw_args = GPUDrawIndirectArgs(ch_vertex_count, 1, ch_first_vertex, ch_packed_xz);
        workgroup_indirect_draw_args[arg_idx] = draw_args;
    }
    workgroupBarrier();

    if (lid.x == 0) {
        let arg_write_count = atomicLoad(&workgroup_indirect_draw_args_count);
        let arg_write_offset = atomicAdd(&indirect_count_buffer[0], arg_write_count);
        for (var i = 0u; i < arg_write_count; i++) {
            indirect_buffer[arg_write_offset + i] = workgroup_indirect_draw_args[i];
        }
    }
}

fn frustum_check_chunk(ch_min: vec3<f32>, ch_max: vec3<f32>) -> bool {
    for (var i = 0; i < 6; i++) {
        let p = camera_view.view_planes[i];

        let pv_x = select(ch_min.x, ch_max.x, p.normal.x >= 0.0);
        let pv_y = select(ch_min.y, ch_max.y, p.normal.y >= 0.0);
        let pv_z = select(ch_min.z, ch_max.z, p.normal.z >= 0.0);

        let d = dot(p.normal, vec3<f32>(pv_x, pv_y, pv_z)) + p.distance;
        if (d < 0.0) {
            return false;
        }
    }
    return true;
}