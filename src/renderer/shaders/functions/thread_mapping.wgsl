
fn thread_index_1d(local_id_x: u32, workgroup_id_x: u32, max_workgroup_x: u32) -> u32 {
    let w_offset = workgroup_id_x * max_workgroup_x;
    let x_offset = local_id_x;
    return w_offset + x_offset;
}