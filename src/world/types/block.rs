use bytemuck::{Pod, Zeroable};
use std::ops::{BitXor, Deref};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct Block(pub u16);

pub trait BlockBytewise {
    const TRANSPARENT_BIT: u16 = 1 << 15;
    fn is_transparent(&self) -> bool;
}

impl BlockBytewise for Block {
    #[inline(always)]
    fn is_transparent(&self) -> bool {
        self.0 & Self::TRANSPARENT_BIT != 0
    }
}

impl BitXor for Block {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Deref for Block {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn blocks_to_u16s<const N: usize>(blocks: [Block; N]) -> [u16; N] {
    unsafe {
        let ptr = &blocks as *const [Block; N];
        let u16_ptr = ptr as *const [u16; N];
        *u16_ptr
    }
}
