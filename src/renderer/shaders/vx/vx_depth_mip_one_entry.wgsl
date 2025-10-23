@group(0) @binding(0)
var src_depth_texture: texture_depth_2d;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_2D, CFG_MAX_WORKGROUP_DIM_2D)
fn depth_mip_one_entry(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= depth_tex_data.mip_w || gid.y >= depth_tex_data.mip_h) {
        return;
    }
    let base_idx = vec2<i32>(gid.xy);
//    let inv_idx = vec2<i32>(
//        bitcast<i32>(gid.x),
//        bitcast<i32>(depth_tex_data.mip_h - 1u - gid.y),
//    );
    let mip_indices = src_indices(base_idx);
    let mip_depth = load_depth_texture_2x2_mip(mip_indices);
    textureStore(dst_texture, base_idx, 0, vec4<f32>(mip_depth, 0.0, 0.0, 0.0));
}

fn load_depth_texture_2x2_mip(mip_indices: MipSrcIndices) -> f32 {
    let tl_texel: f32 = textureLoad(src_depth_texture, mip_indices.tl_idx, 0);
    let tr_texel: f32 = textureLoad(src_depth_texture, mip_indices.tr_idx, 0);
    let bl_texel: f32 = textureLoad(src_depth_texture, mip_indices.bl_idx, 0);
    let br_texel: f32 = textureLoad(src_depth_texture, mip_indices.br_idx, 0);
    return farthest_depth_value(tl_texel, tr_texel, bl_texel, br_texel);
}