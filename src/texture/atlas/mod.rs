use crate::texture::uv::TextureUV;
use crate::texture::{TEXTURE_COUNT, TEXTURE_DIM, Texture};

pub mod helpers;

#[derive(Debug)]
pub struct TextureAtlas {
    pub tiles_per_dim: f32,
    pub dim: f32,
    pub tile_dim: f32,
    pub textures: [TextureUV; TEXTURE_COUNT],
    pub image: image::RgbaImage,
}

impl TextureAtlas {
    fn new() -> Self {
        let tiles_per_dim = (TEXTURE_COUNT as f32).sqrt().ceil();
        let dim = TEXTURE_DIM as f32 * tiles_per_dim;
        let tile_dim = TEXTURE_DIM as f32 / dim;
        Self {
            tiles_per_dim,
            dim,
            tile_dim,
            textures: [TextureUV::default(); TEXTURE_COUNT],
            image: image::RgbaImage::new(dim as u32, dim as u32),
        }
    }

    pub fn uv(&self, texture: Texture) -> &TextureUV {
        &self.textures[texture as usize]
    }
}
