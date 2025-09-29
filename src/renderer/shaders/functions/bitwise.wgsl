fn pack_u16s(a: u32, b: u32) -> u32 {
    return (a & 0xFFFFu) | (b << 16u);
}

fn unpack_u16_low(packed: u32) -> u32 {
    return packed & 0xFFFFu;
}

fn unpack_i16_low(packed: u32) -> i32 {
    let low = unpack_u16_low(packed);
    let low_extended = low | select(0u, 0xFFFF0000u, (low & 0x8000u) != 0u);
    return bitcast<i32>(low_extended);
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

fn unpack_i16s(packed: u32) -> vec2<i32> {
    let u16s = unpack_u16s(packed);
    let low_extended = u16s.x | select(0u, 0xFFFF0000u, (u16s.x & 0x8000u) != 0u);
    let high_extended = u16s.y | select(0u, 0xFFFF0000u, (u16s.y & 0x8000u) != 0u);
    return vec2<i32>(bitcast<i32>(low_extended), bitcast<i32>(high_extended));
}

fn bit_at(value: u32, bit_index: u32) -> u32 {
    return (value >> bit_index) & 1;
}

fn bit_index_mat3x3(x: u32, y: u32, z: u32) -> u32 {
    return (x * 9) + (y * 3) + z;
}