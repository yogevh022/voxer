
fn mask_index(index: u32, mask: u32) -> u32 {
    return mask * (index + VOID_OFFSET);
}