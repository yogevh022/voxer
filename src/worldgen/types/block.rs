use std::ops::Deref;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Block(pub u16);

impl Block {
    const TRANSPARENT_BIT: u16 = 1 << 15;
    #[inline(always)]
    pub fn is_transparent(&self) -> bool {
        self.0 & Self::TRANSPARENT_BIT != 0
    }
}

impl Deref for Block {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
