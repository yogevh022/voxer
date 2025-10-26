fn ilog2(n: u32) -> u32 {
    return 31u - countLeadingZeros(n);
}

fn cot(n: f32) -> f32 {
    return 1.0 / tan(n);
}