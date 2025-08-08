#[derive(Debug, Clone)]
pub struct Block(pub u16);

impl Block {
    const TRANSPARENT_BIT: u16 = 1 << 15;
    #[inline(always)]
    pub fn is_transparent(&self) -> bool {
        self.0 & Self::TRANSPARENT_BIT != 0
    }
}
