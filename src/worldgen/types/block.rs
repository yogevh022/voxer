#[repr(u16)]
#[derive(Debug, Clone)]
pub enum BlockKind {
    Air,
    Dirt,
    Stone,
    Wood,
}

impl BlockKind {
    pub fn is_air(&self) -> bool {
        match self {
            BlockKind::Air => true,
            _ => false,
        }
    }
}
