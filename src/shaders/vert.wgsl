struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Global {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> global: Global;
@group(1) @binding(1)
var<storage, read> model_mats: array<mat4x4<f32>>;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @builtin(instance_index) inst_id: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = global.view_proj * model_mats[inst_id] * vec4<f32>(position, 1.0);
    out.tex_coords = tex_coords;
    return out;
}