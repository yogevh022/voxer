
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
var vx_depth_mipmaps: texture_storage_2d_array<r32float, read>;
@group(0) @binding(6)
var<uniform> vx_camera: VxGPUCamera;

const MAX_WORKGROUP_DRAW_ARGS: u32 = CFG_MAX_WORKGROUP_DIM_1D * 6u;
const MAX_WORKGROUP_MESHING_ARGS: u32 = CFG_MAX_WORKGROUP_DIM_1D + VOID_OFFSET;

var<workgroup> wg_indirect_draw_args: array<GPUDrawIndirectArgs, MAX_WORKGROUP_DRAW_ARGS>;
var<workgroup> wg_indirect_draw_args_count: atomic<u32>;
var<workgroup> wg_indirect_meshing_args: array<GPUChunkMeshEntry, MAX_WORKGROUP_MESHING_ARGS>;
var<workgroup> wg_indirect_meshing_count: atomic<u32>;

var<push_constant> input_length: u32;

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
    let mesh_entry: GPUChunkMeshEntry = chunks_in_view_buffer[mdi_arg_index];
    let chunk_index: u32 = mesh_entry.index;
    let header: GPUVoxelChunkHeader = chunks_buffer[chunk_index].header;
    let chunk_pos = vec3<i32>(header.chunk_x, header.chunk_y, header.chunk_z);
    let chunk_world_pos = vec3<f32>(chunk_pos * i32(CHUNK_DIM));
    let chunk_center_world_pos: vec3<f32> = chunk_world_pos + f32(CHUNK_DIM_HALF);

    let chunk_distance = isquare_distance(camera_chunk_pos, chunk_pos);
    let super_nearby: bool = chunk_distance < 2;
    let within_culling_dist: bool = chunk_distance < bitcast<i32>(culling_distance * culling_distance);
    let within_view: bool = vx_screenspace_sphere_visible(chunk_center_world_pos, CHUNK_BOUNDING_SPHERE_R);
    let exists_mask: u32 = u32(mdi_arg_index < input_length);
    let draw_mask: u32 = exists_mask * u32(within_culling_dist && (super_nearby || within_view));
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
