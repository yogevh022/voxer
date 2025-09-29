use glam::IVec3;

pub fn mix_ivec3_to_u16(v: IVec3) -> u16 {
    // shift and rot constants were picked based on brute force testing
    // 32768 hashes -> 15814 unique
    // 262144 hashes -> 64960 unique

    let mut h = (v.x as u32).wrapping_mul(0x9e3779b9);
    h ^= (v.y as u32).rotate_left(5);
    h ^= (v.z as u32).rotate_right(5);

    h ^= h >> 15;
    h = h.wrapping_mul(0x21f0aaad);
    h ^= h << 1;
    (h >> 2) as u16
}