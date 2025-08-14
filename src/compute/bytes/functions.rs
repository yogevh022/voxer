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
