mod types;
mod meshing;

use crate::render::types::{Mesh, Vertex};
use crate::worldgen::types::{BlockKind, Chunk};
use noise;
use noise::NoiseFn;
use std::sync::Arc;
use crate::texture::helpers::generate_texture_atlas;

const NOISE_SCALE: f64 = 0.05;
pub fn generate_chunk(ns: Arc<impl NoiseFn<f64, 3>>) -> Chunk {
    let blocks: [[[BlockKind; 16]; 16]; 16] = std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            std::array::from_fn(|k| {
                if ns.get([
                    i as f64 * NOISE_SCALE,
                    j as f64 * NOISE_SCALE,
                    k as f64 * NOISE_SCALE,
                ]) > 0.5
                {
                    BlockKind::Stone
                } else {
                    BlockKind::Air
                }
            })
        })
    });
    Chunk { blocks }
}

fn neighbor(blocks: &[[[BlockKind; 16]; 16]; 16], x: isize, y: isize, z: isize) -> Option<&BlockKind> {
    if x < 0 || y < 0 || z < 0 || x >= 16 || y >= 16 || z >= 16 {
        return None
    }
    Some(&blocks[x as usize][y as usize][z as usize])
}

pub fn get_z_mesh(chunk: &Chunk) -> Mesh {
    let mut verts = Vec::new();
    let mut inds = Vec::new();
    let atlas = generate_texture_atlas();
    const chunk_size: usize = 16;
    for x in 0..chunk_size {
        for y in 0..chunk_size {
            for z in 0..chunk_size {
                let pos = (x as f32, y as f32, z as f32);
                if chunk.blocks[x][y][z].is_air() {
                    continue;
                }
                if neighbor(&chunk.blocks, x as isize + 1,y as isize,z as isize).map_or(true, |b| b.is_air()) {
                    meshing::xp_verts(&atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(&chunk.blocks, x as isize - 1,y as isize,z as isize).map_or(true, |b| b.is_air()) {
                    meshing::xm_verts(&atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(&chunk.blocks, x as isize,y as isize + 1,z as isize).map_or(true, |b| b.is_air()) {
                    meshing::yp_verts(&atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(&chunk.blocks, x as isize,y as isize - 1,z as isize).map_or(true, |b| b.is_air()) {
                    meshing::ym_verts(&atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(&chunk.blocks, x as isize,y as isize,z as isize + 1).map_or(true, |b| b.is_air()) {
                    meshing::zp_verts(&atlas, &mut verts, &mut inds, pos);
                }
                if neighbor(&chunk.blocks, x as isize,y as isize,z as isize - 1).map_or(true, |b| b.is_air()) {
                    meshing::zm_verts(&atlas, &mut verts, &mut inds, pos);
                }
            }
        }
    }
    Mesh {
        vertex_offset: verts.len() as u64 * size_of::<Vertex>() as u64,
        index_offset: inds.len() as u64 * size_of::<u16>() as u64,
        vertices: verts,
        indices: inds,
    }
}
