use super::Array3D;
use crate::compute;
use bytemuck::{NoUninit, Pod, Zeroable};
use std::ops::BitXor;

#[inline]
pub fn array_xor<T, const N: usize>(a: &[T; N], b: &[T; N]) -> [T; N]
where
    T: BitXor<Output = T> + Copy + Default + Pod + Zeroable + NoUninit,
{
    let mut faces = [T::default(); N];

    for i in 0..N {
        faces[i] = a[i] ^ b[i];
    }
    faces
}

pub fn rotated_z<T, const X: usize, const Y: usize, const Z: usize>(
    arr_3d: &Array3D<T, X, Y, Z>,
) -> Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    let mut output = Array3D::default();
    for x in 0..X {
        for y in 0..Y {
            for z in 0..Z {
                output[Y - 1 - y][x][z] = arr_3d[x][y][z];
            }
        }
    }
    output
}

pub fn rotated_y<T, const X: usize, const Y: usize, const Z: usize>(
    arr_3d: &Array3D<T, X, Y, Z>,
) -> Array3D<T, X, Y, Z>
where
    T: Copy + Default + Pod + Zeroable + NoUninit,
{
    let mut output = Array3D::default();
    for x in 0..X {
        for y in 0..Y {
            for z in 0..Z {
                output[z][y][X - 1 - x] = arr_3d[x][y][z];
            }
        }
    }
    output
}

#[inline]
pub fn array_pop_count_u16<const N: usize>(arr: [u16; N]) -> u32 {
    arr.into_iter().map(|x| x.count_ones()).sum::<u32>()
}

#[inline]
pub fn array_pop_count_u32<const N: usize>(arr: [u32; N]) -> u32 {
    arr.into_iter().map(|x| x.count_ones()).sum::<u32>()
}