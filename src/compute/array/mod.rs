pub mod functions;
use bytemuck::{NoUninit, Pod, Zeroable};
pub use functions::*;
use std::ops::{Deref, DerefMut};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Array3D<T, const N: usize>(pub [[[T; N]; N]; N])
where
    T: Copy + Default + Pod + Zeroable + NoUninit;
impl<T, const N: usize> From<[[[T; N]; N]; N]> for Array3D<T, N>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    fn from(arr: [[[T; N]; N]; N]) -> Self {
        Self(arr)
    }
}

impl<T, const N: usize> Default for Array3D<T, N>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    fn default() -> Self {
        [[[T::default(); N]; N]; N].into()
    }
}
impl<T, const N: usize> Deref for Array3D<T, N>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    type Target = [[[T; N]; N]; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T, const N: usize> DerefMut for Array3D<T, N>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
