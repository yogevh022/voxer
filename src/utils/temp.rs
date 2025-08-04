use crate::render::types::{Mesh, Model, Vertex};
use crate::texture::{Texture, TextureAtlas};
use crate::types::{SceneObject, Transform};

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

pub fn plane_model_for(ci: &mut u16, texture: Texture, atlas: &TextureAtlas) -> Model {
    let indices = Vec::from([*ci + 0, *ci + 2, *ci + 1, *ci + 0, *ci + 3, *ci + 2]);
    *ci += 4;
    let vertices = quad_verts_for(texture, atlas);
    let mesh = Mesh {
        vertex_offset: (&vertices).len() as u64 * size_of::<Vertex>() as u64,
        vertices: Vec::from(vertices),
        index_offset: (&indices).len() as u64 * size_of::<u16>() as u64,
        indices,
    };
    Model { mesh }
}

pub fn scene_plane(
    curr_index: &mut u16,
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
