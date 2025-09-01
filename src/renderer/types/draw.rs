use bytemuck::{Pod, Zeroable};

// DrawIndexedIndirectArgs, aligned to 32 bytes for cross-backend compatibility
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct DrawIndexedIndirectArgsA32 {
    /// The number of indices to draw.
    pub index_count: u32,
    /// The number of instances to draw.
    pub instance_count: u32,
    /// The first index within the index buffer.
    pub first_index: u32,
    /// The value added to the vertex index before indexing into the vertex buffer.
    pub base_vertex: i32,
    /// The instance ID of the first instance to draw.
    pub first_instance: u32,
    _padding: [u32; 3],
}

impl DrawIndexedIndirectArgsA32 {
    pub fn new(
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) -> Self {
        Self {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
            _padding: [0; 3],
        }
    }
}
