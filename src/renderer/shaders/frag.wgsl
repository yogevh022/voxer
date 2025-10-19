@group(0) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(0) @binding(1)
var atlas_sampler: sampler;

@fragment
fn fs_main(
    @location(0) tex_coords: vec2<f32>,
    @location(1) ao: f32,
) -> @location(0) vec4<f32> {
    let sampled_tex = textureSample(atlas_texture, atlas_sampler, tex_coords);
    return vec4(sampled_tex.rgb * ao, sampled_tex.a);
}
