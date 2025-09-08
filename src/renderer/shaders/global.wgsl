const _KIB: u32 = 1024;
const _MIB: u32 = _KIB * 1024;
const MAX_BUFFER: u32 = 128 * _MIB;

struct Vertex {
    position: vec3<f32>,
    tex_coords: vec2<f32>,
}

alias Index = u32;
alias IndexBuffer = array<Index>;
alias VertexBuffer = array<Vertex>;
