#[inline(always)]
pub fn bit_at(value: u16, index: usize) -> u16 {
    (value >> index) & 1
}
