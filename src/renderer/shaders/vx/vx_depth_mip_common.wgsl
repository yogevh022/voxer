
@group(0) @binding(1)
var dst_texture: texture_storage_2d_array<r32float, write>;

struct DepthTextureData {
    mip_w: u32,
    mip_h: u32,
}
var<push_constant> depth_tex_data: DepthTextureData;

struct MipSrcIndices {
    tl_idx: vec2<i32>,
    tr_idx: vec2<i32>,
    bl_idx: vec2<i32>,
    br_idx: vec2<i32>,
}

fn src_indices(base_idx: vec2<i32>) -> MipSrcIndices {
    var out: MipSrcIndices;
    out.tl_idx = base_idx * vec2<i32>(2);
    out.tr_idx = out.tl_idx + vec2<i32>(1, 0);
    out.bl_idx = out.tl_idx + vec2<i32>(0, 1);
    out.br_idx = out.tl_idx + vec2<i32>(1, 1);
    return out;
}

fn farthest_depth_value(a: f32, b: f32, c: f32, d: f32) -> f32 {
    return max(max(a, b), max(c, d));
}