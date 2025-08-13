
struct GPUChunkEntry {
    vertex_offset: u32,
    index_offset: u32,
    vertex_count: u32,
    index_count: u32,
    slab_index: u32,
    world_position: vec3<f32>,
    blocks: ChunkBlocks,
}

struct Vertex {
    position: vec3<f32>,
    tex_coords: vec2<f32>,
}

struct IndexBuffer {
    indices: array<u32>,
}

struct VertexBuffer {
    vertices: array<Vertex>,
}