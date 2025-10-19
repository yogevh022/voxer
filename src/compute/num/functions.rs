use std::ops;

pub fn mod_complement<T>(a: T, b: T) -> T
where
    T: Copy + ops::Rem<Output = T> + ops::Sub<Output = T>,
{
    (b - (a % b)) % b
}

pub fn ceil_div(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}