pub mod functions;
use bytemuck::{NoUninit, Pod, Zeroable};
pub use functions::*;
use std::ops::{Deref, DerefMut};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Array3D<T, const X: usize, const Y: usize, const Z: usize>(pub [[[T; X]; Y]; Z])
where
    T: Copy + Default + Pod + Zeroable + NoUninit;

impl<T, const X: usize, const Y: usize, const Z: usize> Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    pub fn splat(value: T) -> Self {
        Self([(); Z].map(|_| [(); Y].map(|_| [(); X].map(|_| value))))
    }

    pub fn checkerboard(a: T, b: T) -> Self {
        Self(core::array::from_fn(|z| {
            core::array::from_fn(|y| {
                core::array::from_fn(|x| if (x + y + z) % 2 == 0 { a } else { b })
            })
        }))
    }
}

impl<T, const X: usize, const Y: usize, const Z: usize> From<[[[T; X]; Y]; Z]>
    for Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    fn from(arr: [[[T; X]; Y]; Z]) -> Self {
        Self(arr)
    }
}

impl<T, const X: usize, const Y: usize, const Z: usize> Default for Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    fn default() -> Self {
        [[[T::default(); X]; Y]; Z].into()
    }
}
impl<T, const X: usize, const Y: usize, const Z: usize> Deref for Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    type Target = [[[T; X]; Y]; Z];

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
