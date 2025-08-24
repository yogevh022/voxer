use bytemuck::{Pod, Zeroable};
use std::ops::{BitXor, Deref};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct Block {
    pub value: u16,
}

pub trait BlockBytewise {
    const TRANSPARENT_BIT: u16 = 1 << 15;
    fn is_transparent(&self) -> bool;
}

impl BlockBytewise for Block {
    #[inline(always)]
    fn is_transparent(&self) -> bool {
        self.value & Self::TRANSPARENT_BIT == 0
    }
}

impl BitXor for Block {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value ^ rhs.value,
        }
    }
}

impl Deref for Block {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
