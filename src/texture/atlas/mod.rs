use crate::texture::uv::TextureUV;
use crate::texture::{Texture, TEXTURE_COUNT, TEXTURE_DIM};

pub use self::_TextureAtlas as TextureAtlas;
pub mod helpers;

pub struct _TextureAtlas {
    pub cell_dim: u32,
    pub dim: f32,
    pub textures: [TextureUV; TEXTURE_COUNT],
    pub image: image::RgbaImage,
}

impl _TextureAtlas {
    fn new() -> Self {
        let cell_dim = (TEXTURE_COUNT as f32).sqrt().ceil() as u32;
        let dim = TEXTURE_DIM as f32 * cell_dim as f32;
        Self {
            cell_dim,
            dim,
            textures: [TextureUV::default(); TEXTURE_COUNT],
            image: image::RgbaImage::new(dim as u32, dim as u32),
        }
    }
    
    pub fn uv(&self, texture: Texture) -> &TextureUV {
        &self.textures[texture as usize]
    }
}
