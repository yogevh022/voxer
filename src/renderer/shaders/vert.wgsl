struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) ao: f32,
};

// fixme redundant definition
struct FaceData {
    position__fid__illum__ao: u32,
    // position 12b
    // fid 3b
    // illum 5b
    // ao 8b
    // 4 free
    voxel_type: u32,
    // voxel_type 16b
}

@group(1) @binding(0)
var<uniform> view_projection: mat4x4<f32>;
@group(2) @binding(0)
var<storage, read> chunk_translations_buffer: array<vec3<f32>>;
@group(3) @binding(0)
var<storage, read> face_data_buffer: array<FaceData>;

// fixme temp definition here
const TEMP_VAO_FACTOR: f32 = 0.35;
fn neighbor_count_to_vao(count: u32) -> f32 {
    return 1.0 - (f32(count) * TEMP_VAO_FACTOR);
}
// fixme temp definition
const QUAD_VERTICES = array<array<vec3<f32>, 4>, 6>(
    // +X
    array<vec3<f32>, 4>(
        vec3<f32>(1.0,  1.0,  1.0),
        vec3<f32>(1.0, 1.0,  0.0),
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(1.0,  0.0, 1.0),
    ),
    // -X
    array<vec3<f32>, 4>(
        vec3<f32>(1.0,  1.0,  0.0),
        vec3<f32>(1.0, 1.0,  1.0),
        vec3<f32>(1.0, 0.0, 1.0),
        vec3<f32>(1.0,  0.0, 0.0),
    ),
    // +Y
    array<vec3<f32>, 4>(
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>( 1.0, 1.0, 0.0),
        vec3<f32>( 0.0, 1.0,  0.0),
        vec3<f32>(0.0, 1.0,  1.0),
    ),
    // -Y
    array<vec3<f32>, 4>(
        vec3<f32>(1.0, 1.0,  0.0),
        vec3<f32>( 1.0, 1.0,  1.0),
        vec3<f32>( 0.0, 1.0, 1.0),
        vec3<f32>(0.0, 1.0, 0.0),
    ),
    // +Z
    array<vec3<f32>, 4>(
        vec3<f32>(0.0,  0.0, 1.0),
        vec3<f32>( 1.0,  0.0, 1.0),
        vec3<f32>( 1.0, 1.0, 1.0),
        vec3<f32>(0.0, 1.0, 1.0),
    ),
    // -Z
    array<vec3<f32>, 4>(
        vec3<f32>( 1.0,  0.0, 1.0),
        vec3<f32>(0.0,  0.0, 1.0),
        vec3<f32>(0.0, 1.0, 1.0),
        vec3<f32>( 1.0, 1.0, 1.0),
    ),
);

const QUAD_INDICES = array<u32, 6>(0, 1, 2, 0, 2, 3);

@vertex
fn vs_main(
    @builtin(vertex_index) vid: u32,
    @builtin(instance_index) inst_id: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let face_index = vid / 6;
    let vertex_index = QUAD_INDICES[vid % 6];

    let face_data = face_data_buffer[face_index];
    let pfia = face_data.position__fid__illum__ao;

    let voxel_x: u32 = (pfia >> 8) & 0xF;
    let voxel_y: u32 = (pfia >> 4) & 0xF;
    let voxel_z: u32 = pfia & 0xF;

    let face_id: u32 = (pfia >> 12) & 0x7;

    let illum: u32 = (pfia >> 15) & 0x1F;

    let ao_tl: u32 = (pfia >> 20) & 0x3;
    let ao_tr: u32 = (pfia >> 22) & 0x3;
    let ao_br: u32 = (pfia >> 24) & 0x3;
    let ao_bl: u32 = (pfia >> 26) & 0x3;

    let ao = array<u32, 4>(ao_tl, ao_tr, ao_br, ao_bl);

    let voxel_position = vec3<f32>(f32(voxel_x), f32(voxel_y), f32(voxel_z));
    let quad = QUAD_VERTICES[face_id];
    let vertex_position = voxel_position + quad[vertex_index];
    let chunk_translation = chunk_translations_buffer[inst_id];
    out.position = view_projection * vec4<f32>(chunk_translation + vertex_position, 1.0);
    out.tex_coords = vec2<f32>(0.5, 0.0);
    out.ao = neighbor_count_to_vao(ao[vertex_index]);

    return out;
}
