use crate::compute;
use crate::render::types::{Mesh, Vertex};
use crate::texture::{Texture, TextureAtlas};
use crate::worldgen::types::{CHUNK_DIM, Chunk};
use glam::{Vec2, Vec3};
const X_VERTICES_BY_DIR: [fn(&mut Vec<Vertex>, Vec2, f32, f32, f32, f32); 2] =
    [minus_x_vertices, plus_x_vertices];
const Y_VERTICES_BY_DIR: [fn(&mut Vec<Vertex>, Vec2, f32, f32, f32, f32); 2] =
    [minus_y_vertices, plus_y_vertices];
const Z_VERTICES_BY_DIR: [fn(&mut Vec<Vertex>, Vec2, f32, f32, f32, f32); 2] =
    [minus_z_vertices, plus_z_vertices];

pub fn generate_mesh(chunk: &Chunk, texture_atlas: &TextureAtlas) -> Mesh {
    todo!()
    // let face_count = compute::chunk::chunk_face_count(chunk);
    //
    // let mut vertices = Vec::with_capacity(face_count * 4);
    // let mut indices = Vec::with_capacity(face_count * 6);
    //
    // let x_uv = texture_atlas.uv(Texture::Idk).offset;
    // let y_uv = texture_atlas.uv(Texture::Green).offset;
    // let z_uv = texture_atlas.uv(Texture::Yellow).offset;
    //
    // for i in 0..chunk_faces.x.faces.len() {
    //     let x = (i / CHUNK_SIZE) as f32;
    //     let y = (i % CHUNK_SIZE) as f32;
    //
    //     for z in 0..CHUNK_SIZE {
    //         // if (chunk_faces.x.faces[i] & (1 << z)) != 0 {
    //         //     quad_indices(&mut indices, vertices.len() as u32);
    //         //     let face_dir_bit = ((chunk_faces.x.directions[i] >> z) & 1) as usize;
    //         //     X_VERTICES_BY_DIR[face_dir_bit](&mut vertices, x_uv, x, y, z as f32, texture_atlas.tile_dim);
    //         // }
    //         // if (chunk_faces.y.faces[i] & (1 << z)) != 0 {
    //         //     quad_indices(&mut indices, vertices.len() as u32);
    //         //     let face_dir_bit = ((chunk_faces.y.directions[i] >> z) & 1) as usize;
    //         //     Y_VERTICES_BY_DIR[face_dir_bit](&mut vertices, y_uv, x, y, z as f32, texture_atlas.tile_dim);
    //         // }
    //         if (chunk_faces.z.faces[i] & (1 << z)) != 0 {
    //             quad_indices(&mut indices, vertices.len() as u32);
    //             let face_dir_bit = ((chunk_faces.z.directions[i] >> z) & 1) as usize;
    //             Z_VERTICES_BY_DIR[face_dir_bit](
    //                 &mut vertices,
    //                 z_uv,
    //                 x,
    //                 y,
    //                 z as f32,
    //                 texture_atlas.tile_dim,
    //             );
    //         }
    //     }
    // }
    //
    // Mesh { vertices, indices }
}

fn quad_indices(indices: &mut Vec<u32>, offset: u32) {
    indices.extend([
        offset + 0,
        offset + 1,
        offset + 2,
        offset + 0,
        offset + 2,
        offset + 3,
    ]);
}

fn plus_x_vertices(
    vertices: &mut Vec<Vertex>,
    uv_offset: Vec2,
    x: f32,
    y: f32,
    z: f32,
    tile_dim: f32,
) {
    vertices.extend([
        Vertex {
            position: Vec3::new(x, y + 1.0, z),
            tex_coords: Vec2::new(uv_offset.x, uv_offset.y + tile_dim),
        },
        Vertex {
            position: Vec3::new(x, y, z),
            tex_coords: Vec2::new(uv_offset.x, uv_offset.y),
        },
        Vertex {
            position: Vec3::new(x, y, z + 1.0),
            tex_coords: Vec2::new(uv_offset.x + tile_dim, uv_offset.y),
        },
        Vertex {
            position: Vec3::new(x, y + 1.0, z + 1.0),
            tex_coords: Vec2::new(uv_offset.x + tile_dim, uv_offset.y + tile_dim),
        },
    ]);
}

fn plus_z_vertices(
    vertices: &mut Vec<Vertex>,
    uv_offset: Vec2,
    x: f32,
    y: f32,
    z: f32,
    tile_dim: f32,
) {
    vertices.extend([
        Vertex {
            position: Vec3::new(x, y + 1.0, z + 1.0),
            tex_coords: Vec2::new(uv_offset.x, uv_offset.y),
        },
        Vertex {
            position: Vec3::new(x, y, z + 1.0),
            tex_coords: Vec2::new(uv_offset.x, uv_offset.y + tile_dim),
        },
        Vertex {
            position: Vec3::new(x + 1.0, y, z + 1.0),
            tex_coords: Vec2::new(uv_offset.x + tile_dim, uv_offset.y + tile_dim),
        },
        Vertex {
            position: Vec3::new(x + 1.0, y + 1.0, z + 1.0),
            tex_coords: Vec2::new(uv_offset.x + tile_dim, uv_offset.y),
        },
    ]);
}

fn plus_y_vertices(
    vertices: &mut Vec<Vertex>,
    uv_offset: Vec2,
    x: f32,
    y: f32,
    z: f32,
    tile_dim: f32,
) {
    vertices.extend([
        Vertex {
            position: Vec3::new(x, y + 1.0, z),
            tex_coords: Vec2::new(uv_offset.x, uv_offset.y + tile_dim),
        },
        Vertex {
            position: Vec3::new(x, y + 1.0, z + 1.0),
            tex_coords: Vec2::new(uv_offset.x, uv_offset.y),
        },
        Vertex {
            position: Vec3::new(x + 1.0, y + 1.0, z + 1.0),
            tex_coords: Vec2::new(uv_offset.x + tile_dim, uv_offset.y),
        },
        Vertex {
            position: Vec3::new(x + 1.0, y + 1.0, z),
            tex_coords: Vec2::new(uv_offset.x + tile_dim, uv_offset.y + tile_dim),
        },
    ]);
}

fn minus_x_vertices(
    vertices: &mut Vec<Vertex>,
    uv_offset: Vec2,
    x: f32,
    y: f32,
    z: f32,
    tile_dim: f32,
) {
    vertices.extend([
        Vertex {
            position: Vec3::new(x, y + 1.0, z),
            tex_coords: Vec2::new(uv_offset[0], uv_offset[1] + tile_dim),
        },
        Vertex {
            position: Vec3::new(x, y, z),
            tex_coords: Vec2::new(uv_offset[0], uv_offset[1]),
        },
        Vertex {
            position: Vec3::new(x, y, z + 1.0),
            tex_coords: Vec2::new(uv_offset[0] + tile_dim, uv_offset[1]),
        },
        Vertex {
            position: Vec3::new(x, y + 1.0, z + 1.0),
            tex_coords: Vec2::new(uv_offset[0] + tile_dim, uv_offset[1] + tile_dim),
        },
    ]);
}

fn minus_y_vertices(
    vertices: &mut Vec<Vertex>,
    uv_offset: Vec2,
    x: f32,
    y: f32,
    z: f32,
    tile_dim: f32,
) {
    vertices.extend([
        Vertex {
            position: Vec3::new(x, y, z + 1.0),
            tex_coords: Vec2::new(uv_offset[0], uv_offset[1] + tile_dim),
        },
        Vertex {
            position: Vec3::new(x, y, z),
            tex_coords: Vec2::new(uv_offset[0], uv_offset[1]),
        },
        Vertex {
            position: Vec3::new(x + 1.0, y, z),
            tex_coords: Vec2::new(uv_offset[0] + tile_dim, uv_offset[1]),
        },
        Vertex {
            position: Vec3::new(x + 1.0, y, z + 1.0),
            tex_coords: Vec2::new(uv_offset[0] + tile_dim, uv_offset[1] + tile_dim),
        },
    ]);
}

fn minus_z_vertices(
    vertices: &mut Vec<Vertex>,
    uv_offset: Vec2,
    x: f32,
    y: f32,
    z: f32,
    tile_dim: f32,
) {
    vertices.extend([
        Vertex {
            position: Vec3::new(x + 1.0, y, z),
            tex_coords: Vec2::new(uv_offset[0], uv_offset[1] + tile_dim),
        },
        Vertex {
            position: Vec3::new(x, y, z),
            tex_coords: Vec2::new(uv_offset[0] + tile_dim, uv_offset[1] + tile_dim),
        },
        Vertex {
            position: Vec3::new(x, y + 1.0, z),
            tex_coords: Vec2::new(uv_offset[0] + tile_dim, uv_offset[1]),
        },
        Vertex {
            position: Vec3::new(x + 1.0, y + 1.0, z),
            tex_coords: Vec2::new(uv_offset[0], uv_offset[1]),
        },
    ]);
}
