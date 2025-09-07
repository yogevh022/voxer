#[inline(always)]
pub fn bit_at(value: u16, index: usize) -> u16 {
    (value >> index) & 1
}

const BYTE_UNITS_REPR: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

pub fn repr_bytes(mut value: usize) -> String {
    let mut unit = 0usize;
    while value > 1024 {
        value /= 1024;
        unit += 1;
    }
    let mut repr = value.to_string();
    repr.push_str(BYTE_UNITS_REPR[unit]);
    repr
}

pub fn pack_u32s_to_u64(a: u32, b: u32) -> u64 {
    (a as u64) << 32 | b as u64
}

pub fn unpack_u64_to_u32s(value: u64) -> (u32, u32) {
    (value as u32, (value >> 32) as u32)
}