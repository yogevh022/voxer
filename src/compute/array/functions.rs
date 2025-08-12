use super::Array3D;
use crate::compute;
use std::ops::BitXor;

#[inline]
pub fn xor<T, const N: usize>(a: &[T; N], b: &[T; N]) -> [T; N]
where
    T: BitXor<Output = T> + Default + Copy,
{
    let mut faces = [T::default(); N];

    for i in 0..N {
        faces[i] = a[i] ^ b[i];
    }
    faces
}

pub fn rotated_z<T, const N: usize>(arr_3d: &Array3D<T, N>) -> Array3D<T, N>
where
    T: Copy + Default,
{
    let mut output = Array3D::default();
    for x in 0..N {
        for y in 0..N {
            for z in 0..N {
                output[N - 1 - y][x][z] = arr_3d[x][y][z];
            }
        }
    }
    output
}

pub fn rotated_y<T, const N: usize>(arr_3d: &Array3D<T, N>) -> Array3D<T, N>
where
    T: Copy + Default,
{
    let mut output = Array3D::default();
    for x in 0..N {
        for y in 0..N {
            for z in 0..N {
                output[z][y][N - 1 - x] = arr_3d[x][y][z];
            }
        }
    }
    output
}

pub fn rotated_z_bits<const N: usize>(arr_2d: &[[u16; N]; N]) -> [[u16; N]; N] {
    let mut output = [[0; N]; N];
    for x in 0..N {
        for y in 0..N {
            output[y][N - 1 - x] = arr_2d[x][y];
        }
    }
    output
}

pub fn rotated_y_bits<const N: usize>(arr_2d: &[[u16; N]; N]) -> [[u16; N]; N] {
    let mut output = [[0; N]; N];
    for x in 0..N {
        for y in 0..N {
            for z in 0..N {
                output[z][y] |= compute::bits::bit_at(arr_2d[x][y], z) << (N - 1 - x);
            }
        }
    }
    output
}
