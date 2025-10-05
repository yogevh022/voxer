
@group(1) @binding(0)
var<uniform> camera_view: UniformCameraView;
@group(1) @binding(1)
var<storage, read> face_data_buffer: array<GPUVoxelFaceData>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) ao: f32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vid: u32,
    @builtin(instance_index) packed_xz: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let face_index = vid / 6;
    let vertex_index = QUAD_INDICES[vid % 6];

    let face_data = face_data_buffer[face_index];
    let pfio = face_data.position_fid_illum_ocl;
    let unpacked_xz: vec2<i32> = unpack_i16s(packed_xz);
    let unpacked_y: i32 = unpack_i16_low(face_data.ypos_voxel);

    let chunk_translation = f32(CHUNK_DIM) * vec3<f32>(f32(unpacked_xz.x), f32(unpacked_y), f32(unpacked_xz.y));

    let voxel_x: u32 = (pfio >> 10) & 0x1F;
    let voxel_y: u32 = (pfio >> 5) & 0x1F;
    let voxel_z: u32 = pfio & 0x1F;

    let face_id: u32 = (pfio >> 15) & 0x7;

    let illum: u32 = (pfio >> 18) & 0x1F;

    let ao_tl: u32 = (pfio >> 23) & 0x3;
    let ao_tr: u32 = (pfio >> 25) & 0x3;
    let ao_br: u32 = (pfio >> 27) & 0x3;
    let ao_bl: u32 = (pfio >> 29) & 0x3;

    let ao = array<u32, 4>(ao_tl, ao_tr, ao_br, ao_bl);

    let voxel_position = vec3<f32>(f32(voxel_x), f32(voxel_y), f32(voxel_z)) - 1.0;
    let quad = QUAD_VERTICES[face_id];
    let vertex_position = voxel_position + quad[vertex_index];

    out.position = camera_view.view_proj * vec4<f32>(chunk_translation + vertex_position, 1.0);
    out.tex_coords = TEX_COORDS[vertex_index];
    out.ao = occlusion_count_to_ao(ao[vertex_index]);

    return out;
}
