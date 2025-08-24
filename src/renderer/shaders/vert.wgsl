struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> view_projection: mat4x4<f32>;
@group(2) @binding(0)
var<storage, read> chunk_mmats: array<mat4x4<f32>>;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @builtin(instance_index) instance_id: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = view_projection * chunk_mmats[instance_id] * vec4<f32>(position, 1.0);
//    out.position = view_projection * vec4<f32>(position, 1.0);
    out.tex_coords = tex_coords;
    return out;
}