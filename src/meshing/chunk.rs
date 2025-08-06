use crate::meshing::naive_quad;
use crate::render::types::Mesh;
use crate::texture::TextureAtlas;
use crate::worldgen::types::{BlockKind, Chunk};
use glam::IVec3;
use parking_lot::RwLock;
use std::sync::Arc;
use wgpu::naga::FastHashSet;

pub fn generate_mesh(chunk: &Chunk, texture_atlas: &TextureAtlas) -> Mesh {
    let mut verts = Vec::new();
    let mut inds = Vec::new();

    for x in 0..chunk.blocks.len() {
        for y in 0..chunk.blocks.len() {
            for z in 0..chunk.blocks.len() {
                let pos = (x as f32, y as f32, z as f32);
                if chunk.blocks[x][y][z].is_air() {
                    continue;
                }
                if neighbor(chunk, x as isize + 1, y as isize, z as isize)
                    .map_or(true, |b| b.is_air())
                {
                    naive_quad::plus_x_mesh(texture_atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(chunk, x as isize - 1, y as isize, z as isize)
                    .map_or(true, |b| b.is_air())
                {
                    naive_quad::minus_x_mesh(texture_atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(chunk, x as isize, y as isize + 1, z as isize)
                    .map_or(true, |b| b.is_air())
                {
                    naive_quad::plus_y_mesh(texture_atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(chunk, x as isize, y as isize - 1, z as isize)
                    .map_or(true, |b| b.is_air())
                {
                    naive_quad::minus_y_mesh(texture_atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(chunk, x as isize, y as isize, z as isize + 1)
                    .map_or(true, |b| b.is_air())
                {
                    naive_quad::plus_z_mesh(texture_atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(chunk, x as isize, y as isize, z as isize - 1)
                    .map_or(true, |b| b.is_air())
                {
                    naive_quad::minus_z_mesh(texture_atlas, &mut verts, &mut inds, pos);
                }
            }
        }
    }
    Mesh {
        vertices: verts,
        indices: inds,
    }
}

fn neighbor(chunk: &Chunk, x: isize, y: isize, z: isize) -> Option<&BlockKind> {
    if x < 0
        || y < 0
        || z < 0
        || x >= chunk.blocks.len() as isize
        || y >= chunk.blocks.len() as isize
        || z >= chunk.blocks.len() as isize
    {
        return None;
    }
    Some(&chunk.blocks[x as usize][y as usize][z as usize])
}
