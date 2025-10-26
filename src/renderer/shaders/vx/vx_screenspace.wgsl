
// MUST BE BOUND!
// var vx_depth_mipmaps: texture_storage_2d_array<r32float, read>;
// var<uniform> vx_camera: VxCamera;

var<private> vx__pr_max_depth_mip: u32;
var<private> vx__pr_max_screen_size: u32;

fn vx_screenspace_init() {
    vx__pr_max_screen_size = max(vx_camera.window_size.x, vx_camera.window_size.y);
    vx__pr_max_depth_mip = ilog2(vx__pr_max_screen_size);
}

fn vx_screenspace_sphere_visible(pos: vec3<f32>, radius: f32) -> bool {
    let pos_extended: vec4<f32> = vec4<f32>(pos, 1.0);
    let clip_pos: vec4<f32> = vx_camera.culling_vp * pos_extended;
    let view_pos: vec4<f32> = vx_camera.culling_view * pos_extended;

    let screen = vx__screen_sphere(view_pos, clip_pos, radius);
    let deepest = vx__depth_sample(screen.depth_mipmap);

    let within_screen = !screen.out_of_screen;
    let unoccluded = screen.depth <= deepest;
    return within_screen && unoccluded;
}

struct VxDepthMipMapping {
    level: u32,
    coords: array<vec2<i32>, 4>,
}

struct VxScreenSpaceSphere {
    depth: f32,
    out_of_screen: bool,
    depth_mipmap: VxDepthMipMapping,
}

fn vx__depth_sample(mipmap: VxDepthMipMapping) -> f32 {
    let depth_a: f32 = textureLoad(vx_depth_mipmaps, mipmap.coords[0], mipmap.level).r;
    let depth_b: f32 = textureLoad(vx_depth_mipmaps, mipmap.coords[1], mipmap.level).r;
    let depth_c: f32 = textureLoad(vx_depth_mipmaps, mipmap.coords[2], mipmap.level).r;
    let depth_d: f32 = textureLoad(vx_depth_mipmaps, mipmap.coords[3], mipmap.level).r;
    return max(max(depth_a, depth_b), max(depth_c, depth_d));
}

fn vx__screen_sphere(view_pos: vec4<f32>, clip_pos: vec4<f32>, radius: f32) -> VxScreenSpaceSphere {
    let norm_radius: f32 = vx__normalzied_screen_radius(view_pos, radius);
    let norm_pos: vec2<f32> = vx__normalized_screen_position(clip_pos);

    let screen_r_px: u32 = u32(norm_radius * f32(vx__pr_max_screen_size));
    let mip_level: u32 = min(vx__pr_max_depth_mip - 1, ilog2(screen_r_px * 2u));

    let mip_w: i32 = bitcast<i32>(vx_camera.window_size.x >> mip_level);
    let mip_h: i32 = bitcast<i32>(vx_camera.window_size.y >> mip_level);
    let base_mip_x: i32 = i32((norm_pos.x - norm_radius) * f32(mip_w));
    let base_mip_y: i32 = i32(((1.0 - norm_pos.y) - norm_radius) * f32(mip_h)); // tex y indexing inverted

    let min_mip_x: i32 = max(base_mip_x, 0);
    let max_mip_x: i32 = min(base_mip_x + 1, mip_w);
    let min_mip_y: i32 = max(base_mip_y, 0);
    let max_mip_y: i32 = min(base_mip_y + 1, mip_h);

    let norm_pos_bound: f32 = 1.0 + norm_radius;
    let norm_neg_bound: f32 = -norm_radius;

    let out_of_screen_x: bool = (norm_pos.x < norm_neg_bound) || (norm_pos.x > norm_pos_bound);
    let out_of_screen_y: bool = (norm_pos.y < norm_neg_bound) || (norm_pos.y > norm_pos_bound);

    var depth_mipmap: VxDepthMipMapping;
    depth_mipmap.level = mip_level;
    depth_mipmap.coords = array<vec2<i32>, 4>(
        vec2<i32>(min_mip_x, min_mip_y),
        vec2<i32>(max_mip_x, min_mip_y),
        vec2<i32>(min_mip_x, max_mip_y),
        vec2<i32>(max_mip_x, max_mip_y)
    );

    var screen: VxScreenSpaceSphere;
    screen.depth = vx__clip_depth(view_pos, radius);
    screen.out_of_screen = out_of_screen_x || out_of_screen_y;
    screen.depth_mipmap = depth_mipmap;
    return screen;
}

fn vx__clip_depth(view_pos: vec4<f32>, radius: f32) -> f32 {
    let view_depth: f32 = -view_pos.z;
    let nearest_view_depth: f32 = max(0.0, view_depth - radius);
    let nearest_view_pos: vec4<f32> = vec4<f32>(view_pos.x, view_pos.y, -nearest_view_depth, 1.0);
    let nearest_clip: vec4<f32> = vx_camera.culling_proj * nearest_view_pos;
    return clamp((nearest_clip.z / nearest_clip.w), 0.0, 1.0);
}

fn vx__normalized_screen_position(clip_pos: vec4<f32>) -> vec2<f32> {
    let inv_w: f32 = 1.0 / clip_pos.w;
    return vec2<f32>((clip_pos.xy * inv_w * 0.5) + 0.5);
}

fn vx__normalzied_screen_radius(view_pos: vec4<f32>, radius: f32) -> f32 {
    let view_depth: f32 = -view_pos.z;
    let focal_len_y: f32 = vx_camera.culling_proj[1][1]; // cot(fov_y/2)
    let screen_r: f32 = (radius * focal_len_y) / view_depth;
    return screen_r * 0.5;
}