@group(0) @binding(0)
var src_texture: texture_storage_2d_array<r32float, read>;

@compute @workgroup_size(CFG_MAX_WORKGROUP_DIM_2D, CFG_MAX_WORKGROUP_DIM_2D)
fn depth_mip_x_entry(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= depth_tex_data.mip_w || gid.y >= depth_tex_data.mip_h) {
        return;
    }
    let base_idx = vec2<i32>(gid.xy);
    let mip_indices = src_indices(base_idx);
    let mip_depth = load_storage_texture_2x2_mip(mip_indices);
    textureStore(dst_texture, base_idx, 0, vec4<f32>(mip_depth, 0.0, 0.0, 0.0));
}

fn load_storage_texture_2x2_mip(mip_indices: MipSrcIndices) -> f32 {
    let tl_texel: f32 = textureLoad(src_texture, mip_indices.tl_idx, 0).r;
    let tr_texel: f32 = textureLoad(src_texture, mip_indices.tr_idx, 0).r;
    let bl_texel: f32 = textureLoad(src_texture, mip_indices.bl_idx, 0).r;
    let br_texel: f32 = textureLoad(src_texture, mip_indices.br_idx, 0).r;
    return farthest_depth_value(tl_texel, tr_texel, bl_texel, br_texel);
}