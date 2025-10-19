
@group(0) @binding(0)
var<storage, read_write> indirect_buffer: array<GPUDrawIndirectArgs>;
@group(0) @binding(1)
var<storage, read_write> packed_indirect_buffer: array<GPUPackedIndirectArgsAtomic>;
@group(0) @binding(2)
var<storage, read_write> meshing_batch_buffer: array<GPUChunkMeshEntry>;
@group(0) @binding(3)
var<storage, read> chunks_buffer: array<GPUVoxelChunk>;
@group(0) @binding(4)
var<storage, read> chunks_in_view_buffer: array<GPUChunkMeshEntry>;
@group(0) @binding(5)
var depth_texture_array: texture_storage_2d_array<r32float, read>;
@group(0) @binding(6)
var<uniform> camera_view: UniformCameraView;

const MAX_WORKGROUP_DRAW_ARGS: u32 = CFG_MAX_WORKGROUP_DIM_1D * 6u;
const MAX_WORKGROUP_MESHING_ARGS: u32 = CFG_MAX_WORKGROUP_DIM_1D + VOID_OFFSET;
var<workgroup> wg_indirect_draw_args: array<GPUDrawIndirectArgs, MAX_WORKGROUP_DRAW_ARGS>;
var<workgroup> wg_indirect_draw_args_count: atomic<u32>;
var<workgroup> wg_indirect_meshing_args: array<GPUChunkMeshEntry, MAX_WORKGROUP_MESHING_ARGS>;
var<workgroup> wg_indirect_meshing_count: atomic<u32>;
var<workgroup> wg_max_entries: u32;

var<push_constant> input_length: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn write_culled_mdi(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let camera_position = camera_view.origin.xyz;
    let render_distance_voxels_f32 = camera_view.origin.w;

    let draw_arg_index = thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);
    let mesh_entry = chunks_in_view_buffer[draw_arg_index];
    let chunk_index = mesh_entry.index;

    let chunk_header = chunks_buffer[chunk_index].header;
    let chunk_position_f32 = vec3<f32>(
        f32(chunk_header.chunk_x),
        f32(chunk_header.chunk_y),
        f32(chunk_header.chunk_z),
    );
    let chunk_world_position = chunk_position_f32 * f32(CHUNK_DIM);
    let chunk_world_position_center = chunk_world_position + f32(CHUNK_DIM_HALF);

    let chunk_bounding_sphere_r = f32(CHUNK_DIM_HALF) * 1.9;
    let chunk_clip_position = camera_view.view_proj * vec4<f32>(chunk_world_position_center, 1.0);
    let chunk_ndc = chunk_clip_position.xyz / chunk_clip_position.w;
    let chunk_screen_position = (chunk_ndc.xy * 0.5 + 0.5);
    let chunk_screen_coords = vec2<i32>(chunk_screen_position * vec2<f32>(camera_view.view_dim_px));
    let chunk_depth = (chunk_ndc.z * 0.5 + 0.5);
    let distance_to_chunk_center = distance(camera_position, chunk_world_position_center);
    let px_per_world_unit = (1 / tan(camera_view.fov_y / 2.0)) * (f32(camera_view.view_dim_px.y) / 2.0);
    let pr = (chunk_bounding_sphere_r / chunk_clip_position.z) * px_per_world_unit;
    let depth_mip_level = i32(max(1.0, log2(max(f32(camera_view.view_dim_px.x), f32(camera_view.view_dim_px.y)) / (2 * pr))));

    let mip_depth = textureLoad(depth_texture_array, chunk_screen_coords, depth_mip_level).r;
    let not_occluded = chunk_depth <= mip_depth;

    let fids_facing_camera = chunk_fids_facing_camera(camera_position, chunk_position_f32);
    let face_counts = unpack_mesh_entry_face_counts(mesh_entry);
    let face_offsets = mesh_face_offsets_from(mesh_entry.face_alloc, face_counts);

    let exists = draw_arg_index < input_length;
    let within_render_distance = distance_within_threshold(chunk_world_position, camera_position, render_distance_voxels_f32);
    let within_view_frustum = frustum_check_chunk(chunk_world_position, chunk_world_position + f32(CHUNK_DIM));
    let relevant_mask = u32(exists && within_render_distance && within_view_frustum && not_occluded);
    let requires_meshing_mask = u32(exists) * unpack_mesh_entry_meshing_flag(mesh_entry);

    let mesh_args_idx = atomicAdd(&wg_indirect_meshing_count, requires_meshing_mask);
    wg_indirect_meshing_args[requires_meshing_mask * (mesh_args_idx + VOID_OFFSET)] = mesh_entry;

    for (var fid = 0u; fid < 6u; fid++) {
        let draw_args = GPUDrawIndirectArgs(
            face_counts[fid] * 6u,      // vertex_count
            1u,                         // instance_count
            0u,                         // first_vertex
            face_offsets[fid] * 6u,     // first_instance
        );
        let write_face_mask = relevant_mask * u32(fids_facing_camera[fid] && face_counts[fid] > 0u);
        let draw_args_idx = atomicAdd(&wg_indirect_draw_args_count, write_face_mask);
        wg_indirect_draw_args[write_face_mask * (draw_args_idx + VOID_OFFSET)] = draw_args;
    }
    workgroupBarrier();

    if (lid.x == 0) {
        // write draw args
        let args_count = atomicLoad(&wg_indirect_draw_args_count);
        let args_offset = atomicAdd(&packed_indirect_buffer[0].draw, args_count);
        for (var i = 0u; i < args_count; i++) {
            indirect_buffer[args_offset + i] = wg_indirect_draw_args[VOID_OFFSET + i];
        }

        // write meshing args
        let meshing_args_count = atomicLoad(&wg_indirect_meshing_count);
        let meshing_args_offset = atomicAdd(&packed_indirect_buffer[0].dispatch.x, meshing_args_count);
        for (var i = 0u; i < meshing_args_count; i++) {
            meshing_batch_buffer[meshing_args_offset + i] = wg_indirect_meshing_args[VOID_OFFSET + i];
        }
    }
}

fn chunk_fids_facing_camera(camera_position: vec3<f32>, chunk_position_f32: vec3<f32>) -> array<bool, 6> {
    let camera_chunk_position_f32: vec3<f32> = floor(camera_position / f32(CHUNK_DIM));
    let draw_px = chunk_position_f32.x <= camera_chunk_position_f32.x;
    let draw_mx = chunk_position_f32.x >= camera_chunk_position_f32.x;
    let draw_py = chunk_position_f32.y <= camera_chunk_position_f32.y;
    let draw_my = chunk_position_f32.y >= camera_chunk_position_f32.y;
    let draw_pz = chunk_position_f32.z <= camera_chunk_position_f32.z;
    let draw_mz = chunk_position_f32.z >= camera_chunk_position_f32.z;

    return array<bool, 6>(draw_px, draw_mx, draw_py, draw_my, draw_pz, draw_mz);
}

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