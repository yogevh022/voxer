
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
    @builtin(instance_index) inst_id: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let face_index = vid / 6;
    let vertex_index = QUAD_INDICES[vid % 6];

    let face_data = face_data_buffer[face_index];
    let pfio = face_data.position_fid_illum_ocl;

    let voxel_x: u32 = (pfio >> 8) & 0xF;
    let voxel_y: u32 = (pfio >> 4) & 0xF;
    let voxel_z: u32 = pfio & 0xF;

    let face_id: u32 = (pfio >> 12) & 0x7;

    let illum: u32 = (pfio >> 15) & 0x1F;

    let ao_tl: u32 = (pfio >> 20) & 0x3;
    let ao_tr: u32 = (pfio >> 22) & 0x3;
    let ao_br: u32 = (pfio >> 24) & 0x3;
    let ao_bl: u32 = (pfio >> 26) & 0x3;

    let ao = array<u32, 4>(ao_tl, ao_tr, ao_br, ao_bl);

    let voxel_position = vec3<f32>(f32(voxel_x), f32(voxel_y), f32(voxel_z));
    let quad = QUAD_VERTICES[face_id];
    let vertex_position = voxel_position + quad[vertex_index];
    let chunk_translation = vec3<f32>(0.0, 0.0, 0.0); // fixme temp

    out.position = camera_view.view_projection * vec4<f32>(chunk_translation + vertex_position, 1.0);
    out.tex_coords = TEX_COORDS[vertex_index];
    out.ao = occlusion_count_to_ao(ao[vertex_index]);

    return out;
}
