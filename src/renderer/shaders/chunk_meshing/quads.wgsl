
fn quad_indices(index_index: u32, vertex_base: u32) {
    index_buffer[index_index + 0u] = vertex_base + 0u;
    index_buffer[index_index + 1u] = vertex_base + 1u;
    index_buffer[index_index + 2u] = vertex_base + 2u;
    index_buffer[index_index + 3u] = vertex_base + 0u;
    index_buffer[index_index + 4u] = vertex_base + 2u;
    index_buffer[index_index + 5u] = vertex_base + 3u;
}

fn minus_x_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer[vertex_index + 0].position = vec3<f32>(x, y - 1.0, z - 1.0);
    vertex_buffer[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer[vertex_index + 1].position = vec3<f32>(x, y, z - 1.0);
    vertex_buffer[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 2].position = vec3<f32>(x, y, z);
    vertex_buffer[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 3].position = vec3<f32>(x, y - 1.0, z);
    vertex_buffer[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);
}

fn plus_x_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer[vertex_index + 0].position = vec3<f32>(x, y - 1.0, z - 1.0);
    vertex_buffer[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer[vertex_index + 1].position = vec3<f32>(x, y - 1.0, z);
    vertex_buffer[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 2].position = vec3<f32>(x, y, z);
    vertex_buffer[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 3].position = vec3<f32>(x, y, z - 1.0);
    vertex_buffer[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);
}

fn plus_y_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer[vertex_index + 0].position = vec3<f32>(x - 1.0, y, z - 1.0);
    vertex_buffer[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer[vertex_index + 1].position = vec3<f32>(x, y, z - 1.0);
    vertex_buffer[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer[vertex_index + 2].position = vec3<f32>(x, y, z);
    vertex_buffer[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 3].position = vec3<f32>(x - 1.0, y, z);
    vertex_buffer[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);
}

fn minus_y_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer[vertex_index + 0].position = vec3<f32>(x - 1.0, y, z - 1.0);
    vertex_buffer[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer[vertex_index + 1].position = vec3<f32>(x - 1.0, y, z);
    vertex_buffer[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 2].position = vec3<f32>(x, y, z);
    vertex_buffer[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 3].position = vec3<f32>(x, y, z - 1.0);
    vertex_buffer[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);
}

fn plus_z_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer[vertex_index + 0].position = vec3<f32>(x - 1.0, y - 1.0, z - 1.0);
    vertex_buffer[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer[vertex_index + 1].position = vec3<f32>(x, y - 1.0, z - 1.0);
    vertex_buffer[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer[vertex_index + 2].position = vec3<f32>(x, y, z - 1.0);
    vertex_buffer[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 3].position = vec3<f32>(x - 1.0, y, z - 1.0);
    vertex_buffer[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);
}

fn minus_z_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer[vertex_index + 0].position = vec3<f32>(x - 1.0, y - 1.0, z - 1.0);
    vertex_buffer[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer[vertex_index + 1].position = vec3<f32>(x - 1.0, y, z - 1.0);
    vertex_buffer[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 2].position = vec3<f32>(x, y, z - 1.0);
    vertex_buffer[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);

    vertex_buffer[vertex_index + 3].position = vec3<f32>(x, y - 1.0, z - 1.0);
    vertex_buffer[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);
}

fn t_init_quad(p: u32, q: u32, x: f32, y: f32, z: f32) {
    vertex_buffer[p + 0].position = vec3<f32>(x + -1,y + -1,z + 0);
    vertex_buffer[p + 0].tex_coords = vec2<f32>(0.0, 0.0);

    vertex_buffer[p + 1].position = vec3<f32>(x + 0,y + -1,z + 0);
    vertex_buffer[p + 1].tex_coords = vec2<f32>(0.0, 0.0);

    vertex_buffer[p + 2].position = vec3<f32>(x + 0,y + 0,z + 0);
    vertex_buffer[p + 2].tex_coords = vec2<f32>(0.0, 0.0);

    vertex_buffer[p + 3].position = vec3<f32>(x + -1,y + 0,z + 0);
    vertex_buffer[p + 3].tex_coords = vec2<f32>(0.0, 0.0);

    index_buffer[q + 0] = p + 0u;
    index_buffer[q + 1] = p + 1u;
    index_buffer[q + 2] = p + 2u;
    index_buffer[q + 3] = p + 0u;
    index_buffer[q + 4] = p + 2u;
    index_buffer[q + 5] = p + 3u;
}