mod atlas;
mod uv;
pub use atlas::TextureAtlas;

#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum Texture {
    Yellow,
    Green,
    Idk,
    Murica,
    __Count,
}

pub struct TextureData {
    pub(crate) kind: Texture,
    pub(crate) source: &'static str,
}

pub const TEXTURE_DIM: u32 = 16;
pub const TEXTURE_COUNT: usize = Texture::__Count as usize;

pub static TEXTURES: &[TextureData] = &[
    TextureData {
        kind: Texture::Yellow,
        source: "yellow.png",
    },
    TextureData {
        kind: Texture::Green,
        source: "green.png",
    },
    TextureData {
        kind: Texture::Idk,
        source: "idk.png",
    },
    TextureData {
        kind: Texture::Murica,
        source: "murica.png",
    },
];
