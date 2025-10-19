
@group(0) @binding(0)
var src_depth_texture: texture_depth_2d;
@group(0) @binding(1)
var dst_texture: texture_storage_2d_array<r32float, write>;

var<push_constant> mip_size: vec2<u32>;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_2D, CFG_MAX_WORKGROUP_DIM_2D)
fn depth_mip1_entry(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= mip_size.x || gid.y >= mip_size.y) {
        return;
    }

    let base_idx = vec2<i32>(gid.xy);
    let tl_idx = base_idx * vec2<i32>(2);
    let tr_idx = tl_idx + vec2<i32>(1, 0);
    let bl_idx = tl_idx + vec2<i32>(0, 1);
    let br_idx = tl_idx + vec2<i32>(1, 1);

    let tl_texel = textureLoad(src_depth_texture, tl_idx, 0);
    let tr_texel = textureLoad(src_depth_texture, tr_idx, 0);
    let bl_texel = textureLoad(src_depth_texture, bl_idx, 0);
    let br_texel = textureLoad(src_depth_texture, br_idx, 0);

    let new_depth = max(max(tl_texel, tr_texel), max(bl_texel, br_texel));

    textureStore(dst_texture, base_idx, 0, vec4<f32>(new_depth, 0.0, 0.0, 0.0));
}