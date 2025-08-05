use crate::render::types::{Index, Mesh, Model, Vertex};
use crate::texture::{Texture, TextureAtlas};
use crate::types::{SceneObject, Transform};
use glam::{Vec2, Vec3};

pub fn quad_verts_for(texture: Texture, atlas: &TextureAtlas) -> [Vertex; 4] {
    let uv_offset = atlas.uv(texture).offset;
    [
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.0),
            tex_coords: Vec2::new(uv_offset[0], uv_offset[1] + atlas.tile_dim),
        },
        Vertex {
            position: Vec3::new(0.5, -0.5, 0.0),
            tex_coords: Vec2::new(uv_offset[0] + atlas.tile_dim, uv_offset[1] + atlas.tile_dim),
        },
        Vertex {
            position: Vec3::new(0.5, 0.5, 0.0),
            tex_coords: Vec2::new(uv_offset[0] + atlas.tile_dim, uv_offset[1]),
        },
        Vertex {
            position: Vec3::new(-0.5, 0.5, 0.0),
            tex_coords: Vec2::new(uv_offset[0], uv_offset[1]),
        },
    ]
}

pub fn plane_model_for(ci: &mut Index, texture: Texture, atlas: &TextureAtlas) -> Model {
    let indices = Vec::from([*ci + 0, *ci + 2, *ci + 1, *ci + 0, *ci + 3, *ci + 2]);
    *ci += 4;
    let vertices = quad_verts_for(texture, atlas);
    let mesh = Mesh {
        vertices: Vec::from(vertices),
        indices,
    };
    Model { mesh }
}

pub fn scene_plane(
    curr_index: &mut Index,
    atlas: &TextureAtlas,
    texture: Texture,
    transform: Transform,
) -> SceneObject {
    let model = plane_model_for(curr_index, texture, atlas);
    SceneObject { model, transform }
}

pub fn get_atlas_image() -> image::RgbaImage {
    let atlas_image =
        image::open("src/texture/images/atlas.png").expect("failed to load atlas.png");
    atlas_image.to_rgba8()
}
