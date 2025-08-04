use rand::seq::SliceRandom;
use crate::render::types::Vertex;
use crate::texture::{Texture, TextureAtlas};

fn random_tex() -> Texture {
    let mut rng = rand::thread_rng();
    *[Texture::Idk, Texture::Murica, Texture::Green, Texture::Yellow]
        .choose(&mut rng)
        .unwrap()
}

fn tex(n: u8) -> Texture {
    match n {
        0 => Texture::Idk,
        1 => Texture::Yellow,
        2 => Texture::Green,
        _ => panic!("Invalid texture id"),
    }
}

pub(crate) fn zp_verts(
    atlas: &TextureAtlas,
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u16>,
    pos: (f32, f32, f32),
) {
    let (x, y, z) = pos;
    let uv_offset = atlas.uv(tex(0)).offset;
    let ind_offset = verts.len() as u16;
    verts.extend([
        Vertex {
            position: [x, y + 1.0, z + 1.0],
            tex_coords: [uv_offset[0], uv_offset[1]],
        },
        Vertex {
            position: [x, y, z + 1.0],
            tex_coords: [uv_offset[0], uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x + 1.0, y, z + 1.0],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x + 1.0, y + 1.0, z + 1.0],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1]],
        },

    ]);
    inds.extend([
        ind_offset + 0,
        ind_offset + 1,
        ind_offset + 2,
        ind_offset + 0,
        ind_offset + 2,
        ind_offset + 3,
    ]);
}

pub(crate) fn zm_verts(
    atlas: &TextureAtlas,
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u16>,
    pos: (f32, f32, f32),
) {
    let (x, y, z) = pos;
    let uv_offset = atlas.uv(tex(0)).offset;
    let ind_offset = verts.len() as u16;

    verts.extend([
        Vertex {
            position: [x + 1.0, y, z],
            tex_coords: [uv_offset[0], uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x, y, z],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x, y + 1.0, z],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1]],
        },
        Vertex {
            position: [x + 1.0, y + 1.0, z],
            tex_coords: [uv_offset[0], uv_offset[1]],
        },
    ]);

    inds.extend([
        ind_offset + 0,
        ind_offset + 1,
        ind_offset + 2,
        ind_offset + 0,
        ind_offset + 2,
        ind_offset + 3,
    ]);
}

pub(crate) fn xp_verts(
    atlas: &TextureAtlas,
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u16>,
    pos: (f32, f32, f32),
) {
    let uv_offset = atlas.uv(tex(1)).offset;
    let (x, y, z) = pos;
    let ind_offset = verts.len() as u16;
    verts.extend([
        Vertex {
            position: [x + 1.0, y + 1.0, z + 1.0],
            tex_coords: [uv_offset[0], uv_offset[1]],
        },
        Vertex {
            position: [x + 1.0, y, z + 1.0],
            tex_coords: [uv_offset[0], uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x + 1.0, y, z],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x + 1.0, y + 1.0, z],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1]],
        },
    ]);
    inds.extend([
        ind_offset + 0,
        ind_offset + 1,
        ind_offset + 2,
        ind_offset + 0,
        ind_offset + 2,
        ind_offset + 3,
    ]);
}

pub(crate) fn xm_verts(
    atlas: &TextureAtlas,
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u16>,
    pos: (f32, f32, f32),
) {
    let uv_offset = atlas.uv(tex(1)).offset;
    let (x, y, z) = pos;
    let ind_offset = verts.len() as u16;
    verts.extend([
        Vertex {
            position: [x, y + 1.0, z],
            tex_coords: [uv_offset[0], uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x, y, z],
            tex_coords: [uv_offset[0], uv_offset[1]],
        },
        Vertex {
            position: [x, y, z + 1.0],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1]],
        },
        Vertex {
            position: [x, y + 1.0, z + 1.0],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1] + atlas.tile_dim],
        },
    ]);
    inds.extend([
        ind_offset + 0,
        ind_offset + 1,
        ind_offset + 2,
        ind_offset + 0,
        ind_offset + 2,
        ind_offset + 3,
    ]);
}

pub(crate) fn yp_verts(
    atlas: &TextureAtlas,
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u16>,
    pos: (f32, f32, f32),
) {
    let (x, y, z) = pos;
    let uv_offset = atlas.uv(tex(2)).offset;
    let ind_offset = verts.len() as u16;

    verts.extend([
        Vertex {
            position: [x, y + 1.0, z],
            tex_coords: [uv_offset[0], uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x, y + 1.0, z + 1.0],
            tex_coords: [uv_offset[0], uv_offset[1]],
        },
        Vertex {
            position: [x + 1.0, y + 1.0, z + 1.0],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1]],
        },
        Vertex {
            position: [x + 1.0, y + 1.0, z],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1] + atlas.tile_dim],
        },
    ]);

    inds.extend([
        ind_offset + 0, ind_offset + 1, ind_offset + 2,
        ind_offset + 0, ind_offset + 2, ind_offset + 3,
    ]);
}

pub(crate) fn ym_verts(
    atlas: &TextureAtlas,
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u16>,
    pos: (f32, f32, f32),
) {
    let (x, y, z) = pos;
    let uv_offset = atlas.uv(tex(2)).offset;
    let ind_offset = verts.len() as u16;

    // Counter-clockwise winding when viewed from below
    verts.extend([
        Vertex {
            position: [x, y, z + 1.0],
            tex_coords: [uv_offset[0], uv_offset[1] + atlas.tile_dim],
        },
        Vertex {
            position: [x, y, z],
            tex_coords: [uv_offset[0], uv_offset[1]],
        },
        Vertex {
            position: [x + 1.0, y, z],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1]],
        },
        Vertex {
            position: [x + 1.0, y, z + 1.0],
            tex_coords: [uv_offset[0] + atlas.tile_dim, uv_offset[1] + atlas.tile_dim],
        },
    ]);

    inds.extend([
        ind_offset + 0, ind_offset + 1, ind_offset + 2,
        ind_offset + 0, ind_offset + 2, ind_offset + 3,
    ]);
}