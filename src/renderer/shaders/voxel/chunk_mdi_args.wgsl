
@group(0) @binding(0)
var<storage, read_write> indirect_draw_buffer: array<GPUDrawIndirectArgs>;
@group(0) @binding(1)
var<storage, read_write> indirect_dispatch_buffer: GPUPackedIndirectArgsAtomic;
@group(0) @binding(2)
var<storage, read_write> meshing_batch_buffer: array<GPUChunkMeshEntry>;
@group(0) @binding(3)
var<storage, read> chunks_meta_buffer: array<GPUVoxelChunkHeader>;
@group(0) @binding(4)
var<storage, read_write> chunks_view_buffer: array<GPUChunkMeshEntry>;
@group(0) @binding(5)
var vx_depth_mipmaps: texture_storage_2d_array<r32float, read>;
@group(0) @binding(6)
var<uniform> vx_camera: VxGPUCamera;

const MAX_WORKGROUP_DRAW_ARGS: u32 = CFG_MAX_WORKGROUP_DIM_1D * 6u;
const MAX_WORKGROUP_MESHING_ARGS: u32 = CFG_MAX_WORKGROUP_DIM_1D + VOID_OFFSET;

var<workgroup> wg_indirect_draw_args_count: atomic<u32>;
var<workgroup> wg_indirect_meshing_count: atomic<u32>;
var<workgroup> wg_draw_args_offset: u32;
var<workgroup> wg_mesh_args_offset: u32;

var<private> pr_fid_draws: array<FidDraw, 6>;

var<push_constant> input_length: u32;

const SUPER_NEAR: u32 = 2u;
const SUPER_NEAR_SQ: u32 = SUPER_NEAR * SUPER_NEAR; // consider making this hard 3 (remove SUPER_NEAR)

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_1D)
fn write_culled_mdi(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    vx_screenspace_init();
    let camera_pos: vec3<f32> = vx_camera.culling_origin.xyz;
    let culling_distance: u32 = vx_camera.culling_dist;
    let camera_chunk_pos: vec3<i32> = vec3<i32>(floor(camera_pos * INV_CHUNK_DIM));
    let mdi_arg_index: u32 = thread_index_1d(lid.x, wid.x, CFG_MAX_WORKGROUP_DIM_1D);

    if (mdi_arg_index >= input_length) {
        return; // overflow index
    }

    // extract meshing flag from entry and unflag
    let mesh_entry_ref: ptr<storage, GPUChunkMeshEntry, read_write> = &chunks_view_buffer[mdi_arg_index];
    let entry_and_flag = mesh_entry_consume_meshing_flag(*mesh_entry_ref);
    (*mesh_entry_ref) = entry_and_flag.entry;

    let chunk_header: GPUVoxelChunkHeader = chunks_meta_buffer[entry_and_flag.entry.index];
    let chunk_world_center: vec3<f32> = vec3<f32>(chunk_header.position * i32(CHUNK_DIM)) + f32(CHUNK_DIM_HALF);
    let chunk_distance = isquare_distance(camera_chunk_pos, chunk_header.position);

    // condition masks
    let within_no_cull_zone: bool = chunk_distance < bitcast<i32>(SUPER_NEAR_SQ);
    let within_render_distance: bool = chunk_distance < bitcast<i32>(culling_distance * culling_distance);
    let within_view: bool = vx_screenspace_sphere_visible(chunk_world_center, CHUNK_BOUNDING_SPHERE_R);
    let draw_mask: u32 = u32(within_render_distance && (within_no_cull_zone || within_view));
    let mesh_mask: u32 = entry_and_flag.flag;

    update_local_fid_draws(
        entry_and_flag.entry.face_alloc,
        chunk_header,
        camera_chunk_pos,
        draw_mask,
    );
    let mesh_args_idx = atomicAdd(&wg_indirect_meshing_count, mesh_mask);
    workgroupBarrier();

    if (lid.x == 0) {
        // draw
        let draw_args_count = atomicLoad(&wg_indirect_draw_args_count);
        let draw_args_offset = atomicAdd(&indirect_dispatch_buffer.draw, draw_args_count);
        wg_draw_args_offset = draw_args_offset;
        // mesh
        let mesh_args_count = atomicLoad(&wg_indirect_meshing_count);
        let mesh_args_offset = atomicAdd(&indirect_dispatch_buffer.dispatch.x, mesh_args_count);
        wg_mesh_args_offset = mesh_args_offset;
    }
    workgroupBarrier();

    push_draws();
    push_meshings(entry_and_flag.entry, mesh_args_idx, mesh_mask);
}

struct FidDraw {
    args: GPUDrawIndirectArgs,
    draw_index: u32,
    draw: bool,
}

fn update_local_fid_draws(face_alloc: u32, chunk_header: GPUVoxelChunkHeader, camera_chunk_pos: vec3<i32>, draw_mask: u32) {
    let fid_camera_facings = fids_facing_camera(camera_chunk_pos, chunk_header.position);
    let fid_counts: array<u32, 6> = mesh_entry_face_counts(chunk_header.faces_positive, chunk_header.faces_negative);
    let fid_offsets = mesh_entry_face_offsets(face_alloc, fid_counts);

    let packed_xy: u32 = bitcast<u32>((chunk_header.position.x & 0xFFFFF) | ((chunk_header.position.z & 0xFFF) << 20));
    for (var fid = 0u; fid < 6u; fid++) {
        let axis_has_faces: bool = fid_counts[fid] > 0u;
        let axis_faces_camera: bool = fid_camera_facings[fid];
        let fid_draw_mask = draw_mask * u32(axis_has_faces && axis_faces_camera);
        let fid_draw_idx = atomicAdd(&wg_indirect_draw_args_count, fid_draw_mask);
        let draw_args = GPUDrawIndirectArgs(
            fid_counts[fid] * 6u,      // vertex_count
            1u,                        // instance_count
            fid_offsets[fid] * 6u,     // first_vertex
            packed_xy,                 // first_instance
        );
        pr_fid_draws[fid] = FidDraw(draw_args, fid_draw_idx, bool(fid_draw_mask));
    }
}

fn push_draws() {
    let draw_offset = wg_draw_args_offset;
    for (var fid = 0u; fid < 6u; fid++) {
        let fid_draw = pr_fid_draws[fid];
        if (fid_draw.draw) {
            indirect_draw_buffer[draw_offset + fid_draw.draw_index] = fid_draw.args;
        }
    }
}

fn push_meshings(mesh_entry: GPUChunkMeshEntry, mesh_args_idx: u32, mesh_mask: u32) {
    let mesh_offset = wg_mesh_args_offset;
    if (mesh_mask == 1) {
        meshing_batch_buffer[mesh_offset + mesh_args_idx] = mesh_entry;
    }
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
