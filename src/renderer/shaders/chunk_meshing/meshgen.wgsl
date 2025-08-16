
fn quad_indices(draw: bool, index_index: u32, offset: u32) {
    index_buffer.indices[index_index + 0u] = select(
        index_buffer.indices[index_index + 0u],
        offset + 0u,
        draw
    );
    index_buffer.indices[index_index + 1u] = select(
        index_buffer.indices[index_index + 1u],
        offset + 1u,
        draw
    );
    index_buffer.indices[index_index + 2u] = select(
        index_buffer.indices[index_index + 2u],
        offset + 2u,
        draw
    );
    index_buffer.indices[index_index + 3u] = select(
        index_buffer.indices[index_index + 3u],
        offset + 0u,
        draw
    );
    index_buffer.indices[index_index + 4u] = select(
        index_buffer.indices[index_index + 4u],
        offset + 2u,
        draw
    );
    index_buffer.indices[index_index + 5u] = select(
        index_buffer.indices[index_index + 5u],
        offset + 3u,
        draw
    );
}

fn minus_x_vertices(
    draw: bool,
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    let old_v0 = vertex_buffer.vertices[vertex_index + 0];
    let old_v1 = vertex_buffer.vertices[vertex_index + 1];
    let old_v2 = vertex_buffer.vertices[vertex_index + 2];
    let old_v3 = vertex_buffer.vertices[vertex_index + 3];

    let opt_v0 = Vertex(vec3<f32>(x, y, z), vec2<f32>(uv_offset.x, uv_offset.y));
    let opt_v1 = Vertex(vec3<f32>(x, y, z + 1.0), vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM));
    let opt_v2 = Vertex(vec3<f32>(x, y + 1.0, z + 1.0), vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM));
    let opt_v3 = Vertex(vec3<f32>(x, y + 1.0, z), vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y));

    vertex_buffer.vertices[vertex_index + 0].position = select(old_v0.position, opt_v0.position, draw);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = select(old_v0.tex_coords, opt_v0.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 1].position = select(old_v1.position, opt_v1.position, draw);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = select(old_v1.tex_coords, opt_v1.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 2].position = select(old_v2.position, opt_v2.position, draw);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = select(old_v2.tex_coords, opt_v2.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 3].position = select(old_v3.position, opt_v3.position, draw);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = select(old_v3.tex_coords, opt_v3.tex_coords, draw);
}

fn plus_x_vertices(
    draw: bool,
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    let old_v0 = vertex_buffer.vertices[vertex_index + 0];
    let old_v1 = vertex_buffer.vertices[vertex_index + 1];
    let old_v2 = vertex_buffer.vertices[vertex_index + 2];
    let old_v3 = vertex_buffer.vertices[vertex_index + 3];

    let opt_v0 = Vertex(vec3<f32>(x, y, z + 1.0), vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM));
    let opt_v1 = Vertex(vec3<f32>(x, y, z), vec2<f32>(uv_offset.x, uv_offset.y));
    let opt_v2 = Vertex(vec3<f32>(x, y + 1.0, z), vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y));
    let opt_v3 = Vertex(vec3<f32>(x, y + 1.0, z + 1.0), vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM));

    vertex_buffer.vertices[vertex_index + 0].position = select(old_v0.position, opt_v0.position, draw);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = select(old_v0.tex_coords, opt_v0.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 1].position = select(old_v1.position, opt_v1.position, draw);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = select(old_v1.tex_coords, opt_v1.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 2].position = select(old_v2.position, opt_v2.position, draw);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = select(old_v2.tex_coords, opt_v2.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 3].position = select(old_v3.position, opt_v3.position, draw);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = select(old_v3.tex_coords, opt_v3.tex_coords, draw);
}

fn minus_y_vertices(
    draw: bool,
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    let old_v0 = vertex_buffer.vertices[vertex_index + 0];
    let old_v1 = vertex_buffer.vertices[vertex_index + 1];
    let old_v2 = vertex_buffer.vertices[vertex_index + 2];
    let old_v3 = vertex_buffer.vertices[vertex_index + 3];

    let opt_v0 = Vertex(vec3<f32>(x, y, z), vec2<f32>(uv_offset.x, uv_offset.y));
    let opt_v1 = Vertex(vec3<f32>(x, y, z + 1.0), vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM));
    let opt_v2 = Vertex(vec3<f32>(x + 1.0, y, z + 1.0), vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM));
    let opt_v3 = Vertex(vec3<f32>(x + 1.0, y, z), vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y));

    vertex_buffer.vertices[vertex_index + 0].position = select(old_v0.position, opt_v0.position, draw);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = select(old_v0.tex_coords, opt_v0.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 1].position = select(old_v1.position, opt_v1.position, draw);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = select(old_v1.tex_coords, opt_v1.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 2].position = select(old_v2.position, opt_v2.position, draw);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = select(old_v2.tex_coords, opt_v2.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 3].position = select(old_v3.position, opt_v3.position, draw);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = select(old_v3.tex_coords, opt_v3.tex_coords, draw);
}

fn plus_y_vertices(
    draw: bool,
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    let old_v0 = vertex_buffer.vertices[vertex_index + 0];
    let old_v1 = vertex_buffer.vertices[vertex_index + 1];
    let old_v2 = vertex_buffer.vertices[vertex_index + 2];
    let old_v3 = vertex_buffer.vertices[vertex_index + 3];

    let opt_v0 = Vertex(vec3<f32>(x, y, z + 1.0), vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM));
    let opt_v1 = Vertex(vec3<f32>(x, y, z), vec2<f32>(uv_offset.x, uv_offset.y));
    let opt_v2 = Vertex(vec3<f32>(x + 1.0, y, z), vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y));
    let opt_v3 = Vertex(vec3<f32>(x + 1.0, y, z + 1.0), vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM));

    vertex_buffer.vertices[vertex_index + 0].position = select(old_v0.position, opt_v0.position, draw);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = select(old_v0.tex_coords, opt_v0.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 1].position = select(old_v1.position, opt_v1.position, draw);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = select(old_v1.tex_coords, opt_v1.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 2].position = select(old_v2.position, opt_v2.position, draw);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = select(old_v2.tex_coords, opt_v2.tex_coords, draw);

    vertex_buffer.vertices[vertex_index + 3].position = select(old_v3.position, opt_v3.position, draw);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = select(old_v3.tex_coords, opt_v3.tex_coords, draw);
}

fn model_matrix_from_position(position: vec3<f32>) -> mat4x4<f32> {
    var result: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(position, 1.0),
    );
    return result;
}

fn t_init_quad(p: u32, q: u32, x: f32, y: f32, z: f32) {
    vertex_buffer.vertices[p + 0].position = vec3<f32>(x + -1,y + -1,z + 0);
    vertex_buffer.vertices[p + 0].tex_coords = vec2<f32>(0.0, 0.0);

    vertex_buffer.vertices[p + 1].position = vec3<f32>(x + 0,y + -1,z + 0);
    vertex_buffer.vertices[p + 1].tex_coords = vec2<f32>(0.0, 0.0);

    vertex_buffer.vertices[p + 2].position = vec3<f32>(x + 0,y + 0,z + 0);
    vertex_buffer.vertices[p + 2].tex_coords = vec2<f32>(0.0, 0.0);

    vertex_buffer.vertices[p + 3].position = vec3<f32>(x + -1,y + 0,z + 0);
    vertex_buffer.vertices[p + 3].tex_coords = vec2<f32>(0.0, 0.0);

    index_buffer.indices[q + 0] = p + 0u;
    index_buffer.indices[q + 1] = p + 1u;
    index_buffer.indices[q + 2] = p + 2u;
    index_buffer.indices[q + 3] = p + 0u;
    index_buffer.indices[q + 4] = p + 2u;
    index_buffer.indices[q + 5] = p + 3u;
}