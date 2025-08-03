pub fn index_based_entries<const N: usize>(
    resources: [wgpu::BindingResource; N],
) -> [wgpu::BindGroupEntry; N] {
    let mut i = 0;
    resources.map(|r| {
        let entry = wgpu::BindGroupEntry {
            binding: i,
            resource: r,
        };
        i += 1;
        entry
    })
}
