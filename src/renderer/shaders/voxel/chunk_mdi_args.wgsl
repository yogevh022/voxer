
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

var<private> pr_screen_size: u32;
var<private> pr_max_mip_level: u32;

var<push_constant> input_length: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn write_culled_mdi(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let camera_position = camera_view.origin.xyz;
    let render_distance_voxels_f32 = camera_view.origin.w;
    pr_screen_size = max(camera_view.view_dim_px.x, camera_view.view_dim_px.y);
    pr_max_mip_level = u32(log2(f32(pr_screen_size)));

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

    let not_occluded = !occlusion_check_chunk(camera_position, chunk_world_position_center);

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

fn occlusion_check_chunk(camera_position: vec3<f32>, chunk_world_center: vec3<f32>) -> bool {
    let screen_state = chunk_screen_state(camera_position, chunk_world_center);

    let mip_depth = textureLoad(depth_texture_array, screen_state.mip_coords, screen_state.mip_level).r;

    let valid_for_occlusion = screen_state.mip_level < pr_max_mip_level - 2;
    let is_occluded = screen_state.depth > mip_depth;

//    return mip_depth != 0.0;
//    return screen_state.depth == 0.0 && 0.0 == mip_depth;
//    return !!(screen_state.depth < mip_depth);
    return valid_for_occlusion && is_occluded;
}

struct ChunkScreenState {
    mip_coords: vec2<i32>,
    depth: f32,
    mip_level: u32,
}

const CHUNK_BOUNDING_SPHERE_R: f32 = f32(CHUNK_DIM_HALF) * 1.8;
fn chunk_screen_state(camera_position: vec3<f32>, chunk_world_center: vec3<f32>) -> ChunkScreenState {
    let clip = camera_view.view_proj * vec4<f32>(chunk_world_center, 1.0);
    let inv_w = 1.0 / clip.w;
    var ndc_xy = clip.xy * inv_w;
//    let depth = -(clip.z / (clip.w - CHUNK_BOUNDING_SPHERE_R));
    let depth = -(clip.z / clip.w);

    let bounding_sphere_radius_ndc = CHUNK_BOUNDING_SPHERE_R * inv_w;
    let bounding_sphere_screen_px = bounding_sphere_radius_ndc * f32(camera_view.view_dim_px.y);
    let mip_level = pr_max_mip_level - u32(floor(log2(bounding_sphere_screen_px)));

    let normalized_screen_position = ndc_xy * 0.5 + 0.5;
    var mip_coords = vec2<i32>(normalized_screen_position * vec2<f32>(camera_view.view_dim_px));
    let mip_height = bitcast<i32>(camera_view.view_dim_px.y >> mip_level);
    mip_coords.x = mip_coords.x >> mip_level;
    mip_coords.y = mip_height - (mip_coords.y >> mip_level);

    return ChunkScreenState(mip_coords, depth, mip_level);
}