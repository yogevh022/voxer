pub struct TransformResources {
    pub uniform_buffer: wgpu::Buffer,
    pub model_matrix_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
