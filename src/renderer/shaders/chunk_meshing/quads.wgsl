
fn quad_indices(index_index:u32, vertex_base: u32) {
    local_index_array[index_index + 0u] = vertex_base + 0u;
    local_index_array[index_index + 1u] = vertex_base + 1u;
    local_index_array[index_index + 2u] = vertex_base + 2u;
    local_index_array[index_index + 3u] = vertex_base + 0u;
    local_index_array[index_index + 4u] = vertex_base + 2u;
    local_index_array[index_index + 5u] = vertex_base + 3u;
}

fn quad_indices_inversed(index_index:u32, vertex_base: u32) {
    local_index_array[index_index + 0u] = vertex_base + 0u;
    local_index_array[index_index + 1u] = vertex_base + 2u;
    local_index_array[index_index + 2u] = vertex_base + 1u;
    local_index_array[index_index + 3u] = vertex_base + 0u;
    local_index_array[index_index + 4u] = vertex_base + 3u;
    local_index_array[index_index + 5u] = vertex_base + 2u;
}

fn quad_vertices_x(
    vertices_ao: array<f32, 4>,
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    local_vertex_array[vertex_index + 0].position = vec3<f32>(x, y - 1.0, z - 1.0);
    local_vertex_array[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);
    local_vertex_array[vertex_index + 0].ao = vertices_ao[0];

    local_vertex_array[vertex_index + 1].position = vec3<f32>(x, y - 1.0, z);
    local_vertex_array[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);
    local_vertex_array[vertex_index + 1].ao = vertices_ao[1];

    local_vertex_array[vertex_index + 2].position = vec3<f32>(x, y, z);
    local_vertex_array[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
    local_vertex_array[vertex_index + 2].ao = vertices_ao[2];

    local_vertex_array[vertex_index + 3].position = vec3<f32>(x, y, z - 1.0);
    local_vertex_array[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);
    local_vertex_array[vertex_index + 3].ao = vertices_ao[3];
}

fn quad_vertices_y(
    vertices_ao: array<f32, 4>,
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    local_vertex_array[vertex_index + 0].position = vec3<f32>(x - 1.0, y, z - 1.0);
    local_vertex_array[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);
    local_vertex_array[vertex_index + 0].ao = vertices_ao[0];

    local_vertex_array[vertex_index + 1].position = vec3<f32>(x - 1.0, y, z);
    local_vertex_array[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);
    local_vertex_array[vertex_index + 1].ao = vertices_ao[1];

    local_vertex_array[vertex_index + 2].position = vec3<f32>(x, y, z);
    local_vertex_array[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
    local_vertex_array[vertex_index + 2].ao = vertices_ao[2];

    local_vertex_array[vertex_index + 3].position = vec3<f32>(x, y, z - 1.0);
    local_vertex_array[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);
    local_vertex_array[vertex_index + 3].ao = vertices_ao[3];
}

fn quad_vertices_z(
    vertices_ao: array<f32, 4>,
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    local_vertex_array[vertex_index + 0].position = vec3<f32>(x - 1.0, y - 1.0, z);
    local_vertex_array[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);
    local_vertex_array[vertex_index + 0].ao = vertices_ao[0];

    local_vertex_array[vertex_index + 1].position = vec3<f32>(x, y - 1.0, z);
    local_vertex_array[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);
    local_vertex_array[vertex_index + 1].ao = vertices_ao[1];

    local_vertex_array[vertex_index + 2].position = vec3<f32>(x, y, z);
    local_vertex_array[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
    local_vertex_array[vertex_index + 2].ao = vertices_ao[2];

    local_vertex_array[vertex_index + 3].position = vec3<f32>(x - 1.0, y, z);
    local_vertex_array[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);
    local_vertex_array[vertex_index + 3].ao = vertices_ao[3];
}
