use crate::render::types::Mesh;
use crate::texture::TextureAtlas;
use crate::worldgen::meshing;
use crate::worldgen::types::block::BlockKind;

pub const CHUNK_SIZE: usize = 16;
pub struct Chunk {
    pub(crate) blocks: [[[BlockKind; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn generate_mesh(&self, texture_atlas: &TextureAtlas) -> Mesh {
        let mut verts = Vec::new();
        let mut inds = Vec::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let pos = (x as f32, y as f32, z as f32);
                    if self.blocks[x][y][z].is_air() {
                        continue;
                    }
                    if neighbor(&self.blocks, x as isize + 1, y as isize, z as isize)
                        .map_or(true, |b| b.is_air())
                    {
                        meshing::xp_verts(texture_atlas, &mut verts, &mut inds, pos);
                    }
                    if neighbor(&self.blocks, x as isize - 1, y as isize, z as isize)
                        .map_or(true, |b| b.is_air())
                    {
                        meshing::xm_verts(texture_atlas, &mut verts, &mut inds, pos);
                    }
                    if neighbor(&self.blocks, x as isize, y as isize + 1, z as isize)
                        .map_or(true, |b| b.is_air())
                    {
                        meshing::yp_verts(texture_atlas, &mut verts, &mut inds, pos);
                    }
                    if neighbor(&self.blocks, x as isize, y as isize - 1, z as isize)
                        .map_or(true, |b| b.is_air())
                    {
                        meshing::ym_verts(texture_atlas, &mut verts, &mut inds, pos);
                    }
                    if neighbor(&self.blocks, x as isize, y as isize, z as isize + 1)
                        .map_or(true, |b| b.is_air())
                    {
                        meshing::zp_verts(texture_atlas, &mut verts, &mut inds, pos);
                    }
                    if neighbor(&self.blocks, x as isize, y as isize, z as isize - 1)
                        .map_or(true, |b| b.is_air())
                    {
                        meshing::zm_verts(texture_atlas, &mut verts, &mut inds, pos);
                    }
                }
            }
        }
        Mesh {
            vertices: verts,
            indices: inds,
        }
    }
}

fn neighbor(
    blocks: &[[[BlockKind; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    x: isize,
    y: isize,
    z: isize,
) -> Option<&BlockKind> {
    if x < 0
        || y < 0
        || z < 0
        || x >= CHUNK_SIZE as isize
        || y >= CHUNK_SIZE as isize
        || z >= CHUNK_SIZE as isize
    {
        return None;
    }
    Some(&blocks[x as usize][y as usize][z as usize])
}
