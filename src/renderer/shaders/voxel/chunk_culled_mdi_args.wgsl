
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

var<workgroup> wg_indirect_draw_args: array<GPUDrawIndirectArgs, CFG_MAX_WORKGROUP_DIM_1D>;
var<workgroup> wg_indirect_draw_args_count: atomic<u32>;
var<workgroup> wg_max_entries: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn write_culled_mdi(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    if (lid.x == 0) {
        atomicStore(&wg_indirect_draw_args_count, 0u);
        wg_max_entries = chunks_in_view_buffer[0].index;
    }
    workgroupBarrier();

    let camera_position = camera_view.origin.xyz;
    let render_distance_voxels_f32 = camera_view.origin.w;

    let draw_arg_index = 1u + thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);
    let mesh_entry = chunks_in_view_buffer[draw_arg_index];
    let chunk_index = mesh_entry.index;

    // fixme move some vars to workgroup
    let chunk_header = chunks_buffer[chunk_index].header;
    let chunk_position_f32 = vec3<f32>(
        f32(chunk_header.chunk_x),
        f32(chunk_header.chunk_y),
        f32(chunk_header.chunk_z),
    );
    let chunk_world_position = chunk_position_f32 * f32(CHUNK_DIM);

    let visible_fids = visible_face_ids(camera_position, chunk_position_f32);
    let exists = draw_arg_index <= wg_max_entries;
    let within_render_distance = distance_within_threshold(chunk_world_position, camera_position, render_distance_voxels_f32);
    let within_view_frustum = frustum_check_chunk(chunk_world_position, chunk_world_position + f32(CHUNK_DIM));

    if true || (exists && within_render_distance && within_view_frustum) {
        let face_counts = unpack_mesh_face_counts(mesh_entry);
        let face_offsets = mesh_face_counts_to_offsets(face_counts);
        var arg_idx = atomicAdd(&wg_indirect_draw_args_count, visible_fids.count);
        for (var fid = 0u; fid < 6u; fid++) {
            if (visible_fids.draw_fid[fid]) {
                let draw_args = GPUDrawIndirectArgs(
                    face_counts[fid] * 6u,                              // vertex_count
                    1u,                                                 // instance_count
                    0u,                                                 // first_vertex
                    (mesh_entry.face_alloc + face_offsets[fid]) * 6u,   // first_instance
                );
                wg_indirect_draw_args[arg_idx] = draw_args;
                arg_idx++;
            }
        }
    }
    workgroupBarrier();

    if (lid.x == 0) {
        let arg_write_count = atomicLoad(&wg_indirect_draw_args_count);
        let arg_write_offset = atomicAdd(&indirect_count_buffer[0], arg_write_count);
        for (var i = 0u; i < arg_write_count; i++) {
            indirect_buffer[arg_write_offset + i] = wg_indirect_draw_args[i];
        }
    }
}

struct VisibleFaceIDs {
    draw_fid: array<bool, 6>,
    count: u32,
}

fn visible_face_ids(camera_position: vec3<f32>, chunk_position_f32: vec3<f32>) -> VisibleFaceIDs {
    let camera_chunk_position_f32: vec3<f32> = floor(camera_position / f32(CHUNK_DIM));
    let draw_px = chunk_position_f32.x <= camera_chunk_position_f32.x;
    let draw_mx = chunk_position_f32.x >= camera_chunk_position_f32.x;
    let draw_py = chunk_position_f32.y <= camera_chunk_position_f32.y;
    let draw_my = chunk_position_f32.y >= camera_chunk_position_f32.y;
    let draw_pz = chunk_position_f32.z <= camera_chunk_position_f32.z;
    let draw_mz = chunk_position_f32.z >= camera_chunk_position_f32.z;

//    let masks = array<bool, 6>(draw_px, draw_mx, draw_py, draw_my, draw_pz, draw_mz);
    let draw_fid = array<bool, 6>(false, false, true, false, false, false);
//    let count = u32(draw_px) + u32(draw_mx) + u32(draw_py) + u32(draw_my) + u32(draw_pz) + u32(draw_mz);
    let count = 1u;

    return VisibleFaceIDs(draw_fid, count);
}

// fixme move to modular place
fn distance_within_threshold(a: vec3<f32>,b: vec3<f32>, threshold: f32) -> bool {
    let threshold_sq = threshold * threshold;
    let distance_sq = dot(a - b, a - b);
    return distance_sq < threshold_sq;
}

fn frustum_check_chunk(chunk_world_min: vec3<f32>, chunk_world_max: vec3<f32>) -> bool {
    for (var i = 0; i < 6; i++) {
        let plane = camera_view.view_planes[i];

        let pv = vec3<f32>(
            select(chunk_world_min.x, chunk_world_max.x, plane.equation.x >= 0.0),
            select(chunk_world_min.y, chunk_world_max.y, plane.equation.y >= 0.0),
            select(chunk_world_min.z, chunk_world_max.z, plane.equation.z >= 0.0),
        );

        if (dot(plane.equation.xyz, pv) + plane.equation.w < 0.0) {
            return false;
        }
    }
    return true;
}