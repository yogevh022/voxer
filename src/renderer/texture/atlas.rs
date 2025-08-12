use crate::renderer::texture::uv::TextureUV;
use crate::renderer::texture::{TEXTURES, Texture, TextureData};
use glam::Vec2;

#[derive(Debug)]
pub struct TextureAtlas<const D: usize, const N: usize> {
    pub tiles_per_dim: f32,
    pub dim: f32,
    pub tile_dim: f32,
    pub textures: [TextureUV; N],
    pub image: image::RgbaImage,
}

impl<const D: usize, const N: usize> TextureAtlas<D, N> {
    fn generate() -> Self {
        let tiles_per_dim = (N as f32).sqrt().ceil();
        let dim = D as f32 * tiles_per_dim;
        let tile_dim = D as f32 / dim;
        let mut atlas = Self {
            tiles_per_dim,
            dim,
            tile_dim,
            textures: [TextureUV::default(); N],
            image: image::RgbaImage::new(dim as u32, dim as u32),
        };
        for tex_data in TEXTURES.iter() {
            atlas.write_texture(tex_data);
        }
        atlas
    }

    pub fn uv(&self, texture: Texture) -> &TextureUV {
        &self.textures[texture as usize]
    }
}

impl<const D: usize, const N: usize> TextureAtlas<D, N> {
    fn write_texture(&mut self, tex_data: &TextureData) {
        let tex_image = image::open(get_image_path(tex_data.source))
            .unwrap()
            .to_rgba8();
        let tex_index = tex_data.kind as u32 as f32;
        let x_offset = D as f32 * (tex_index % self.tiles_per_dim);
        let y_offset = D as f32 * (tex_index / self.tiles_per_dim).floor();
        self.textures[tex_index as usize].offset =
            Vec2::new(x_offset / self.dim, y_offset / self.dim);
        for (x, y, pixel) in tex_image.enumerate_pixels() {
            self.image
                .put_pixel(x + x_offset as u32, y + y_offset as u32, *pixel);
        }
    }
}

fn get_image_path(img_src: &'static str) -> String {
    "src/texture/images/".to_string() + img_src
}
