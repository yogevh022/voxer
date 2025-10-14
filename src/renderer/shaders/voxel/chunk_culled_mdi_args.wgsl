
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

var<workgroup> workgroup_indirect_draw_args: array<GPUDrawIndirectArgs, CFG_MAX_WORKGROUP_DIM_1D>;
var<workgroup> workgroup_indirect_draw_args_count: atomic<u32>;
var<workgroup> workgroup_max_entries: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn write_culled_mdi(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    if (lid.x == 0) {
        atomicStore(&workgroup_indirect_draw_args_count, 0);
        workgroup_max_entries = chunks_in_view_buffer[0].index;
    }
    workgroupBarrier();

    let draw_arg_index = 1 + thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);

    let mesh_entry = chunks_in_view_buffer[draw_arg_index];
    let chunk_index = mesh_entry.index;
    let chunk_vertex_count = mesh_entry.face_count * 6u;
    let chunk_vertex_offset = mesh_entry.face_alloc * 6u;

    let chunk_header = chunks_buffer[chunk_index].header;
    let chunk_position = vec3<f32>(
        f32(chunk_header.chunk_x),
        f32(chunk_header.chunk_y),
        f32(chunk_header.chunk_z),
    );
    let chunk_world_min = chunk_position * f32(CHUNK_DIM);
    let chunk_world_max = chunk_world_min + f32(CHUNK_DIM);

    let chunk_exists = draw_arg_index <= workgroup_max_entries;
    let chunk_within_render_distance = distance_check_chunk(chunk_world_min);
    let chunk_within_frustum = frustum_check_chunk(chunk_world_min, chunk_world_max);

    if chunk_exists && chunk_within_render_distance && chunk_within_frustum {
        let arg_idx = atomicAdd(&workgroup_indirect_draw_args_count, 1);
        const INSTANCE_COUNT = 1u;
        const FIRST_VERTEX = 0u;
        let draw_args = GPUDrawIndirectArgs(
            chunk_vertex_count,
            INSTANCE_COUNT,
            FIRST_VERTEX,
            chunk_vertex_offset
        );
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

fn distance_check_chunk(chunk_world_min: vec3<f32>) -> bool {
    let render_distance_sq = camera_view.origin.w * camera_view.origin.w;
    let camera_origin = camera_view.origin.xyz;
    let chunk_distance_sq = dot(camera_origin - chunk_world_min, camera_origin - chunk_world_min);
    return chunk_distance_sq < render_distance_sq;
}

fn frustum_check_chunk(chunk_world_min: vec3<f32>, chunk_world_max: vec3<f32>) -> bool {
    for (var i = 0; i < 6; i++) {
        let p = camera_view.view_planes[i];

        let pv = vec3<f32>(
            select(chunk_world_min.x, chunk_world_max.x, p.equation.x >= 0.0),
            select(chunk_world_min.y, chunk_world_max.y, p.equation.y >= 0.0),
            select(chunk_world_min.z, chunk_world_max.z, p.equation.z >= 0.0)
        );

        if (dot(p.equation.xyz, pv) + p.equation.w < 0.0) {
            return false;
        }
    }
    return true;
}