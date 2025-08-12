pub mod functions;
use bytemuck::{Pod, Zeroable};
pub use functions::*;
use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Array3D<T, const N: usize>(pub [[[T; N]; N]; N])
where
    T: Copy + Default;
impl<T, const N: usize> From<[[[T; N]; N]; N]> for Array3D<T, N>
where
    T: Copy + Default,
{
    fn from(arr: [[[T; N]; N]; N]) -> Self {
        Self(arr)
    }
}

impl<T, const N: usize> Default for Array3D<T, N>
where
    T: Copy + Default,
{
    fn default() -> Self {
        [[[T::default(); N]; N]; N].into()
    }
}
impl<T, const N: usize> Deref for Array3D<T, N>
where
    T: Copy + Default,
{
    type Target = [[[T; N]; N]; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T, const N: usize> DerefMut for Array3D<T, N>
where
    T: Copy + Default,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
