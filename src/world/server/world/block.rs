use bytemuck::{Pod, Zeroable};
use std::ops::{BitXor, Deref};

const VOXEL_TRANSPARENT_BIT: u16 = 1 << 15;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct VoxelBlock {
    pub value: u16,
}

impl VoxelBlock {
    pub const EMPTY: Self = Self { value: 0 };

    pub fn is_transparent(&self) -> bool {
        self.value & VOXEL_TRANSPARENT_BIT == 0
    }
}

impl BitXor for VoxelBlock {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value ^ rhs.value,
        }
    }
}

impl Deref for VoxelBlock {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
