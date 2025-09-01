use std::ops;

pub fn mod_complement<T>(a: T, b: T) -> T
where
    T: Copy + ops::Rem<Output = T> + ops::Sub<Output = T>,
{
    (b - (a % b)) % b
}
