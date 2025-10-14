
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
    let vertex_index = QUAD_INDICES[current_vertex % 6];

    let face_data = face_data_buffer[face_index];
    let face_voxel = unpack_face_voxel(face_data);
    let face_position = vec3<f32>(unpack_face_position(face_data));
    let face_lighting = unpack_face_lighting(face_data);
    let face_ao = unpack_face_ao(face_data);

    let vertex_position = face_position + QUAD_VERTICES[face_voxel.face_id][vertex_index];

//    out.position = vec4<f32>(f32(current_vertex),f32(vertex_index),1.0,1.0);
    out.position = camera_view.view_proj * vec4<f32>(vertex_position, 1.0);
    out.tex_coords = TEX_COORDS[vertex_index];
    out.ao = occlusion_count_to_ao(face_ao[vertex_index]);

    return out;
}

struct VoxelFace {
    voxel: u32,
    face_id: u32,
}

fn unpack_face_voxel(face_data: GPUVoxelFaceData) -> VoxelFace {
    let voxel = unpack_u16_low(face_data.word_e);
    let face_id = (face_data.word_e >> 28) & 0x7;
    return VoxelFace(voxel, face_id);
}

fn unpack_face_position(face_data: GPUVoxelFaceData) -> vec3<i32> {
    return vec3<i32>(
        unpack_i24_low(face_data.word_a),
        unpack_i12_low(face_data.word_c),
        unpack_i24_low(face_data.word_b),
    );
}

fn unpack_face_lighting(face_data: GPUVoxelFaceData) -> array<vec3<u32>, 4> {
    let top_left_lighting = vec3<u32>(
        (face_data.word_a >> 24) & 0x3F,
        (face_data.word_d >> 18) & 0x3F,
        (face_data.word_d >> 24) & 0x3F,
    );

    let top_right_lighting = vec3<u32>(
        (face_data.word_b >> 24) & 0x3F,
        (face_data.word_e >> 16) & 0x3F,
        (face_data.word_e >> 22) & 0x3F,
    );

    let bot_left_lighting = vec3<u32>(
        (face_data.word_c >> 12) & 0x3F,
        (face_data.word_c >> 18) & 0x3F,
        (face_data.word_c >> 24) & 0x3F,
    );

    let bot_right_lighting = vec3<u32>(
        (face_data.word_d) & 0x3F,
        (face_data.word_d >> 6) & 0x3F,
        (face_data.word_d >> 12) & 0x3F,
    );

    return array<vec3<u32>, 4>(
        top_left_lighting,
        top_right_lighting,
        bot_left_lighting,
        bot_right_lighting
    );
}

fn unpack_face_ao(face_data: GPUVoxelFaceData) -> array<u32, 4> {
    let tl_ao = face_data.word_a >> 30;
    let tr_ao = face_data.word_b >> 30;
    let bl_ao = face_data.word_c >> 30;
    let br_ao = face_data.word_d >> 30;
    return array<u32, 4>(tl_ao, tr_ao, br_ao, bl_ao);
}
