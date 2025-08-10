use super::Array3D;

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
