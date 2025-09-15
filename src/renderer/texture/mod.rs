mod atlas;


const TEXTURE_DIM: u32 = 16;
const TEXTURE_COUNT: usize = TextureKind::__Count as usize;

#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum TextureKind {
    Yellow,
    Green,
    Idk,
    Murica,
    __Count,
}

struct TextureData {
    pub(crate) kind: TextureKind,
    pub(crate) source: &'static str,
}

static TEXTURES: &[TextureData] = &[
    TextureData {
        kind: TextureKind::Yellow,
        source: "yellow.png",
    },
    TextureData {
        kind: TextureKind::Green,
        source: "green.png",
    },
    TextureData {
        kind: TextureKind::Idk,
        source: "idk.png",
    },
    TextureData {
        kind: TextureKind::Murica,
        source: "murica.png",
    },
];
