use crate::texture::atlas::TextureAtlas;
use crate::texture::{TEXTURE_DIM, TEXTURES, TextureData};

fn get_image_path(img_src: &'static str) -> String {
    "src/texture/images/".to_string() + img_src
}

pub fn write_texture_to_atlas(tex_data: &TextureData, atlas: &mut TextureAtlas) {
    let tex_image = image::open(get_image_path(tex_data.source))
        .unwrap()
        .to_rgba8();
    let tex_index = tex_data.kind as u32 as f32;
    let x_offset = TEXTURE_DIM as f32 * (tex_index % atlas.tiles_per_dim);
    let y_offset = TEXTURE_DIM as f32 * (tex_index / atlas.tiles_per_dim).floor();
    atlas.textures[tex_index as usize].offset = [x_offset / atlas.dim, y_offset / atlas.dim];
    for (x, y, pixel) in tex_image.enumerate_pixels() {
        atlas
            .image
            .put_pixel(x + x_offset as u32, y + y_offset as u32, *pixel);
    }
}

pub fn generate_texture_atlas() -> TextureAtlas {
    let mut atlas = TextureAtlas::new();
    for tex_data in TEXTURES.iter() {
        write_texture_to_atlas(tex_data, &mut atlas)
    }
    atlas
}
