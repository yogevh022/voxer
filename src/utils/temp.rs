use crate::render::types::Vertex;
use crate::texture::{Texture, TextureAtlas};

pub fn quad_verts_for(texture: Texture, atlas: &TextureAtlas) -> [Vertex; 4] {
    let uv_offset = atlas.uv(texture).offset;
    [
        Vertex {
            position: [-0.5, -0.5, 0.0],
            tex_coords: [uv_offset[0], uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [0.5, 0.5, 0.0],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1]],
        },
        Vertex {
            position: [-0.5, 0.5, 0.0],
            tex_coords: [uv_offset[0], uv_offset[1]],
        },
    ]
}

pub fn get_atlas_image() -> image::RgbaImage {
    let atlas_image =
        image::open("src/texture/images/atlas.png").expect("failed to load atlas.png");
    atlas_image.to_rgba8()
}
