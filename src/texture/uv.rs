#[derive(Debug, Copy, Clone)]
pub struct TextureUV {
    pub(crate) offset: [f32; 2],
}

impl Default for TextureUV {
    fn default() -> Self {
        Self { offset: [0.0, 0.0] }
    }
}
