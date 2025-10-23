
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
var<uniform> vx_camera: VxCamera;

const MAX_WORKGROUP_DRAW_ARGS: u32 = CFG_MAX_WORKGROUP_DIM_1D * 6u;
const MAX_WORKGROUP_MESHING_ARGS: u32 = CFG_MAX_WORKGROUP_DIM_1D + VOID_OFFSET;

var<workgroup> wg_indirect_draw_args: array<GPUDrawIndirectArgs, MAX_WORKGROUP_DRAW_ARGS>;
var<workgroup> wg_indirect_draw_args_count: atomic<u32>;
var<workgroup> wg_indirect_meshing_args: array<GPUChunkMeshEntry, MAX_WORKGROUP_MESHING_ARGS>;
var<workgroup> wg_indirect_meshing_count: atomic<u32>;

var<private> pr_max_screen_size: u32;
var<private> pr_max_depth_mip: u32;
var<private> pr_culling_distance: u32;

var<push_constant> input_length: u32;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn write_culled_mdi(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let camera_pos: vec3<f32> = vx_camera.culling_origin.xyz;
    let camera_chunk_pos: vec3<i32> = vec3<i32>(floor(camera_pos * INV_CHUNK_DIM));
    let mdi_arg_index: u32 = thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);
    let mesh_entry: GPUChunkMeshEntry = chunks_in_view_buffer[mdi_arg_index];
    let chunk_index: u32 = mesh_entry.index;
    let header: GPUVoxelChunkHeader = chunks_buffer[chunk_index].header;
    let chunk_pos = vec3<i32>(header.chunk_x, header.chunk_y, header.chunk_z);
    let chunk_world_pos = vec3<f32>(chunk_pos * i32(CHUNK_DIM));

    pr_culling_distance = vx_camera.culling_dist;
    pr_max_screen_size = max(vx_camera.window_size.x, vx_camera.window_size.y);
    pr_max_depth_mip = ilog2(pr_max_screen_size);

    // fixme remove vf logic ?
//    let within_view_frustum: bool = is_chunk_in_frustum(chunk_world_pos, chunk_world_pos + f32(CHUNK_DIM));

    let chunk_distance = isquare_distance(camera_chunk_pos, chunk_pos);
    let within_culling_dist: bool = chunk_distance < bitcast<i32>(pr_culling_distance * pr_culling_distance);
    let super_nearby: bool = chunk_distance < 2;
    let within_view: bool = super_nearby || is_chunk_visible(chunk_world_pos);
    let exists_mask: u32 = u32(mdi_arg_index < input_length);
    let draw_mask: u32 = exists_mask * u32(within_culling_dist && within_view);
    let mesh_mask: u32 = exists_mask * unpack_mesh_entry_meshing_flag(mesh_entry);

    push_to_draw_batch(mesh_entry, camera_chunk_pos, chunk_pos, draw_mask);
    push_to_meshing_batch(mesh_entry, mesh_mask);
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

fn push_to_draw_batch(mesh_entry: GPUChunkMeshEntry, camera_chunk_pos: vec3<i32>, chunk_pos: vec3<i32>, draw_mask: u32) {
    let fids_facing_camera = fids_facing_camera(camera_chunk_pos, chunk_pos);
    let face_counts = unpack_mesh_entry_face_counts(mesh_entry);
    let face_offsets = mesh_face_offsets_from(mesh_entry.face_alloc, face_counts);

    for (var fid = 0u; fid < 6u; fid++) {
        let draw_args = GPUDrawIndirectArgs(
            face_counts[fid] * 6u,      // vertex_count
            1u,                         // instance_count
            0u,                         // first_vertex
            face_offsets[fid] * 6u,     // first_instance
        );
        let has_faces: bool = face_counts[fid] > 0u;
        let facing_camera: bool = fids_facing_camera[fid];
        let draw_fid_mask = draw_mask * u32(has_faces && facing_camera);
        let draw_args_idx = atomicAdd(&wg_indirect_draw_args_count, draw_fid_mask);
        let draw_args_idx_masked = mask_index(draw_args_idx, draw_fid_mask);
        wg_indirect_draw_args[draw_args_idx_masked] = draw_args;
    }
}

fn push_to_meshing_batch(mesh_entry: GPUChunkMeshEntry, mesh_mask: u32) {
    let mesh_args_idx = atomicAdd(&wg_indirect_meshing_count, mesh_mask);
    let mesh_args_idx_masked = mask_index(mesh_args_idx, mesh_mask);
    wg_indirect_meshing_args[mesh_args_idx_masked] = mesh_entry;
}

fn fids_facing_camera(camera_chunk_pos: vec3<i32>, chunk_pos: vec3<i32>) -> array<bool, 6> {
    let draw_px = chunk_pos.x <= camera_chunk_pos.x;
    let draw_mx = chunk_pos.x >= camera_chunk_pos.x;
    let draw_py = chunk_pos.y <= camera_chunk_pos.y;
    let draw_my = chunk_pos.y >= camera_chunk_pos.y;
    let draw_pz = chunk_pos.z <= camera_chunk_pos.z;
    let draw_mz = chunk_pos.z >= camera_chunk_pos.z;

    return array<bool, 6>(draw_px, draw_mx, draw_py, draw_my, draw_pz, draw_mz);
}

fn is_chunk_in_frustum(chunk_world_min: vec3<f32>, chunk_world_max: vec3<f32>) -> bool {
    for (var i = 0; i < 6; i++) {
        let plane = vx_camera.culling_vf[i];
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

fn is_chunk_visible(chunk_world_pos: vec3<f32>) -> bool {
    let chunk_center_world_pos: vec3<f32> = chunk_world_pos + f32(CHUNK_DIM_HALF);
    let screen: ScreenChunk = screen_chunk(chunk_center_world_pos);
    let mip_level: u32 = screen.mip_level;

    let mip_depth_a: f32 = textureLoad(depth_texture_array, screen.depth_mip_coords[0], mip_level).r;
    let mip_depth_b: f32 = textureLoad(depth_texture_array, screen.depth_mip_coords[1], mip_level).r;
    let mip_depth_c: f32 = textureLoad(depth_texture_array, screen.depth_mip_coords[2], mip_level).r;
    let mip_depth_d: f32 = textureLoad(depth_texture_array, screen.depth_mip_coords[3], mip_level).r;

    let deepest: f32 = max(max(mip_depth_a, mip_depth_b), max(mip_depth_c, mip_depth_d));

//    let close = deepest - screen.depth <= 0.01 || screen.depth == 1.0;
//    return close;

    return (!screen.out_of_screen) && screen.depth <= deepest;
}

fn screen_chunk(chunk_center_world_pos: vec3<f32>) -> ScreenChunk {
    let chunk_center_vec4: vec4<f32> = vec4<f32>(chunk_center_world_pos, 1.0);
    let clip_pos: vec4<f32> = vx_camera.culling_vp * chunk_center_vec4;
    let view_pos: vec4<f32> = vx_camera.culling_view * chunk_center_vec4;
    return chunk_depth_mip_coords(view_pos, clip_pos);
}

struct ScreenChunk {
    mip_level: u32,
    depth: f32,
    out_of_screen: bool,
    depth_mip_coords: array<vec2<i32>, 4>,
}

fn chunk_depth_mip_coords(view_pos: vec4<f32>, clip_pos: vec4<f32>) -> ScreenChunk {
    const CHUNK_BOUNDING_SPHERE_R: f32 = f32(CHUNK_DIM_HALF) * 1.75;
    let norm_radius: f32 = normalzied_screen_radius(view_pos, CHUNK_BOUNDING_SPHERE_R);
    let norm_pos: vec2<f32> = normalized_screen_position(clip_pos);

    let screen_r_px: u32 = u32(norm_radius * f32(pr_max_screen_size));
    let mip_level: u32 = min(pr_max_depth_mip - 1, ilog2(screen_r_px * 2u));

    let mip_w: i32 = bitcast<i32>(vx_camera.window_size.x >> mip_level);
    let mip_h: i32 = bitcast<i32>(vx_camera.window_size.y >> mip_level);
    let base_mip_x: i32 = i32((norm_pos.x - norm_radius) * f32(mip_w));
    let base_mip_y: i32 = i32((norm_pos.y - norm_radius) * f32(mip_h));

    let min_mip_x: i32 = max(base_mip_x, 0);
    let max_mip_x: i32 = clamp(base_mip_x + 1, 0, mip_w - 1);
    let min_mip_y: i32 = max(base_mip_y, 0);
    let max_mip_y: i32 = clamp(base_mip_y + 1, 0, mip_h - 1);

//    let base_mip_x: u32 = u32(min(1.0, norm_pos.x + norm_radius) * f32(window_size.x)) >> mip_level;
//    let base_mip_y: u32 = u32(min(1.0, norm_pos.y + norm_radius) * f32(window_size.y)) >> mip_level;
//    let min_mip_x: i32 = bitcast<i32>(base_mip_x);
//    let max_mip_x: i32 = bitcast<i32>(max(base_mip_x - 1u, 0u));
//    let min_mip_y: i32 = bitcast<i32>(base_mip_y);
//    let max_mip_y: i32 = bitcast<i32>(max(base_mip_y - 1u, 0u));

    let norm_pos_bound: f32 = 1.0 + norm_radius;
    let norm_neg_bound: f32 = -norm_radius;

    let out_of_screen_x: bool = (norm_pos.x < norm_neg_bound) || (norm_pos.x > norm_pos_bound);
    let out_of_screen_y: bool = (norm_pos.y < norm_neg_bound) || (norm_pos.y > norm_pos_bound);

    var screen: ScreenChunk;
    screen.mip_level = mip_level;
    screen.depth = clip_depth(view_pos, CHUNK_BOUNDING_SPHERE_R);
    screen.out_of_screen = out_of_screen_x || out_of_screen_y;
    screen.depth_mip_coords = array<vec2<i32>, 4>(
        vec2<i32>(min_mip_x, min_mip_y),
        vec2<i32>(max_mip_x, min_mip_y),
        vec2<i32>(min_mip_x, max_mip_y),
        vec2<i32>(max_mip_x, max_mip_y)
    );
    return screen;
}

fn clip_depth(view_pos: vec4<f32>, radius: f32) -> f32 {
    let view_depth: f32 = -view_pos.z;
    let nearest_view_depth: f32 = max(0.0, view_depth - radius);
    let nearest_view_pos: vec4<f32> = vec4<f32>(view_pos.x, view_pos.y, -nearest_view_depth, 1.0);
    let nearest_clip: vec4<f32> = vx_camera.culling_proj * nearest_view_pos;
    return clamp((nearest_clip.z / nearest_clip.w), 0.0, 1.0);
}

fn normalized_screen_position(clip_pos: vec4<f32>) -> vec2<f32> {
    let inv_w: f32 = 1.0 / clip_pos.w;
    return vec2<f32>((clip_pos.xy * inv_w * 0.5) + 0.5);
}

fn normalzied_screen_radius(view_pos: vec4<f32>, radius: f32) -> f32 {
    let view_depth: f32 = -view_pos.z;
    let focal_len_y: f32 = vx_camera.culling_proj[1][1]; // cot(fov_y/2)
    let screen_r: f32 = (radius * focal_len_y) / view_depth;
    return screen_r * 0.5;
}