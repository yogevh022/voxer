
@group(0) @binding(0) var depth_tex: texture_2d_array<f32>;
@group(0) @binding(1) var depth_sampler: sampler;

@fragment
fn dbg_fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = textureSample(depth_tex, depth_sampler, in.uv, 0).r;

    let gamma = 0.02;
    let mapped = 1.0 - pow(1.0 - d, gamma);
    return vec4<f32>(mapped, mapped, mapped, 1.0);
}
