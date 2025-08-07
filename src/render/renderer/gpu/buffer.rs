use bytemuck::NoUninit;

pub fn reorder_to_indices<A: NoUninit, const N: usize, F: Fn(usize) -> [A; N]>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    step_size: usize,
    indices: &[(usize, usize)],
    get_value: F,
) {
    for i in indices {
        queue.write_buffer(
            buffer,
            (i.0 * step_size) as u64,
            bytemuck::cast_slice(&get_value(i.1)),
        )
    }
}
