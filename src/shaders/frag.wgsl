@group(0) @binding(0)
var _tex: texture_2d<f32>;

@group(0) @binding(1)
var _sampler: sampler;

@fragment
fn fs_main(
    @location(0) tex_coords: vec2<f32>
) -> @location(0) vec4<f32> {
    let sampled_tex = textureSample(_tex, _sampler, tex_coords);
    return sampled_tex;
}