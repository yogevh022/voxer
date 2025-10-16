
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
    @builtin(vertex_index) draw_vertex_index: u32,
    @builtin(instance_index) base_vertex: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let current_vertex = base_vertex + draw_vertex_index;
    let face_index = current_vertex / 6;
    let vertex_index = QUAD_INDICES[draw_vertex_index % 6];

    let face_data = face_data_buffer[face_index];
    let face_voxel = unpack_face_voxel(face_data);
    let face_position = vec3<f32>(unpack_face_position(face_data));
    let face_lighting = unpack_face_lighting(face_data);
    let face_ao = unpack_face_ao(face_data);

    let vertex_position = face_position + QUAD_VERTICES[face_voxel.face_id][vertex_index];

    out.position = camera_view.view_proj * vec4<f32>(vertex_position, 1.0);
    out.tex_coords = TEX_COORDS[vertex_index];
    out.ao = occlusion_count_to_ao(face_ao[vertex_index]);

    return out;
}
