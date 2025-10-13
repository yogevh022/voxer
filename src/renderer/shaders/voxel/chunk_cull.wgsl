
@group(0) @binding(0)
var<storage, read_write> indirect_buffer: array<GPUDrawIndirectArgs>;
@group(0) @binding(1)
var<storage, read_write> indirect_count_buffer: array<atomic<u32>>;
@group(0) @binding(2)
var<storage, read> chunks_buffer: array<GPUVoxelChunk>;
@group(0) @binding(3)
var<storage, read> chunks_in_view_buffer: array<GPUChunkMeshEntry>;
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
    let draw_arg_index = 1 + wg_offset + x_offset;

    if (lid.x == 0) {
        atomicStore(&workgroup_indirect_draw_args_count, 0);
        workgroup_max_entries = chunks_in_view_buffer[0].index;
    }
    workgroupBarrier();

    let ch_idx = chunks_in_view_buffer[draw_arg_index].index;
    let ch_vertex_count = chunks_in_view_buffer[draw_arg_index].face_count * 6u;
    let ch_vertex_offset = chunks_in_view_buffer[draw_arg_index].face_offset * 6u;

    let ch_position = chunks_buffer[ch_idx].header.position;
    let ch_world_min = vec3<f32>(ch_position) * f32(CHUNK_DIM);
    let ch_world_max = ch_world_min + f32(CHUNK_DIM);

    if (draw_arg_index <= workgroup_max_entries) && (true || frustum_check_chunk(ch_world_min, ch_world_max)) {
        let arg_idx = atomicAdd(&workgroup_indirect_draw_args_count, 1);
        let draw_args = GPUDrawIndirectArgs(ch_vertex_count, 1, 0, ch_vertex_offset);
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

fn frustum_check_chunk(ch_world_min: vec3<f32>, ch_world_max: vec3<f32>) -> bool {
    for (var i = 0; i < 6; i++) {
        let p = camera_view.view_planes[i];

        let pv = vec3<f32>(
            select(ch_world_min.x, ch_world_max.x, p.normal.x >= 0.0),
            select(ch_world_min.y, ch_world_max.y, p.normal.y >= 0.0),
            select(ch_world_min.z, ch_world_max.z, p.normal.z >= 0.0)
        );

        if (dot(p.normal, pv) + p.distance < 0.0) {
            return false;
        }
    }
    return true;
}