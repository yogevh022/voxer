fn pack_u16s(a: u32, b: u32) -> u32 {
    return (a & 0xFFFFu) | (b << 16u);
}

fn unpack_u16_low(packed: u32) -> u32 {
    return packed & 0xFFFFu;
}

fn unpack_u16_high(packed: u32) -> u32 {
    return packed >> 16u;
}

fn get_u16(packed: u32, index: u32) -> u32 {
    return select(packed >> 16u, packed & 0xFFFFu, index == 0u);
}

fn unpack_u16s(packed: u32) -> vec2<u32> {
    return vec2<u32>(packed & 0xFFFFu, packed >> 16u);
}

fn bit_at(value: u32, bit_index: u32) -> u32 {
    return (value >> bit_index) & 1;
}

fn bit_index_mat3x3(x: u32, y: u32, z: u32) -> u32 {
    return (x * 9) + (y * 3) + z;
}