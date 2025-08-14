pub mod functions;
use bytemuck::{NoUninit, Pod, Zeroable};
pub use functions::*;
use std::ops::{Deref, DerefMut};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Array3D<T, const X: usize, const Y: usize, const Z: usize>(pub [[[T; Z]; Y]; X])
where
    T: Copy + Default + Pod + Zeroable + NoUninit;
impl<T, const X: usize, const Y: usize, const Z: usize> From<[[[T; Z]; Y]; X]>
    for Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    fn from(arr: [[[T; Z]; Y]; X]) -> Self {
        Self(arr)
    }
}

impl<T, const X: usize, const Y: usize, const Z: usize> Default for Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    fn default() -> Self {
        [[[T::default(); Z]; Y]; X].into()
    }
}
impl<T, const X: usize, const Y: usize, const Z: usize> Deref for Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    type Target = [[[T; Z]; Y]; X];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T, const X: usize, const Y: usize, const Z: usize> DerefMut for Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
