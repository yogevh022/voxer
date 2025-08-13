
fn quad_indices(index_index: u32, offset: u32) {
    index_buffer.indices[index_index + 0u] = offset + 0u;
    index_buffer.indices[index_index + 1u] = offset + 1u;
    index_buffer.indices[index_index + 2u] = offset + 2u;
    index_buffer.indices[index_index + 3u] = offset + 0u;
    index_buffer.indices[index_index + 4u] = offset + 2u;
    index_buffer.indices[index_index + 5u] = offset + 3u;
}

fn plus_x_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer.vertices[vertex_index + 0].position = vec3<f32>(x, y + 1.0, z);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer.vertices[vertex_index + 1].position = vec3<f32>(x, y, z);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 2].position = vec3<f32>(x, y, z + 1.0);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 3].position = vec3<f32>(x, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

fn minus_x_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer.vertices[vertex_index + 0].position = vec3<f32>(x, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer.vertices[vertex_index + 1].position = vec3<f32>(x, y, z + 1.0);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 2].position = vec3<f32>(x, y, z);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 3].position = vec3<f32>(x, y + 1.0, z);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

fn plus_y_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer.vertices[vertex_index + 0].position = vec3<f32>(x, y + 1.0, z);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer.vertices[vertex_index + 1].position = vec3<f32>(x, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 2].position = vec3<f32>(x + 1.0, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 3].position = vec3<f32>(x, y + 1.0, z + 1.0);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}

fn minus_y_vertices(
    vertex_index: u32,
    uv_offset: vec2<f32>,
    x: f32,
    y: f32,
    z: f32,
) {
    vertex_buffer.vertices[vertex_index + 0].position = vec3<f32>(x, y, z + 1.0);
    vertex_buffer.vertices[vertex_index + 0].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y + TILE_DIM);

    vertex_buffer.vertices[vertex_index + 1].position = vec3<f32>(x, y, z);
    vertex_buffer.vertices[vertex_index + 1].tex_coords = vec2<f32>(uv_offset.x, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 2].position = vec3<f32>(x + 1.0, y, z);
    vertex_buffer.vertices[vertex_index + 2].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y);

    vertex_buffer.vertices[vertex_index + 3].position = vec3<f32>(x + 1.0, y, z + 1.0);
    vertex_buffer.vertices[vertex_index + 3].tex_coords = vec2<f32>(uv_offset.x + TILE_DIM, uv_offset.y + TILE_DIM);
}



fn face_mask_for_axis(axis: u32, x: u32, y: u32, z: u32) -> FaceMask {
//        let face_bit = (chunk_face[0u][x].faces[y][z] >> ((16u << n) - 1u)) & 1u;
//        let dir_bit = (chunk_face[0u][x].dirs[y][z] >> ((16u << n) - 1u)) & 1u;
        return FaceMask(1u, 1u);
}

fn write_faces_x(
    index_offset: u32,
    index_count: ptr<function, u32>,
    vertex_offset: u32,
    vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.5, 0.5);

    let x_f32 = f32(x);
    let y_f32 = f32(y);
    let z_f32 = f32(z);

    // logic for both u16s packed into the u32
    for (var n = 0u; n < 2u; n += 1u) {
        let face_mask = face_mask_for_axis(0u, x, y, z);
        let i_masked = (VOID_REF_OFFSET + index_offset + (*index_count)) * face_mask.face_bit;
        let v_masked = (VOID_REF_OFFSET + vertex_offset + (*vertex_count)) * face_mask.face_bit;

        quad_indices(i_masked, v_masked);
        plus_x_vertices(v_masked * face_mask.dir_bit, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_x_vertices(v_masked * (1u ^ face_mask.dir_bit), temp_uv, x_f32, y_f32, z_f32);
        (*index_count) += (6u * face_mask.face_bit);
        (*vertex_count) += (4u * face_mask.face_bit);
    }
}

fn write_faces_y(
    index_offset: u32,
    index_count: ptr<function, u32>,
    vertex_offset: u32,
    vertex_count: ptr<function, u32>,
    x: u32,
    y: u32,
    z: u32,
) {
    let temp_uv: vec2<f32> = vec2<f32>(0.5, 0.5);

    let x_f32 = f32(x);
    let y_f32 = f32(y);
    let z_f32 = f32(z);

    // logic for both u16s packed into the u32
    for (var n = 0u; n < 2u; n += 1u) {
        let face_mask = face_mask_for_axis(1u, x, y, z);
        let i_masked = (VOID_REF_OFFSET + index_offset + (*index_count)) * face_mask.face_bit;
        let v_masked = (VOID_REF_OFFSET + vertex_offset + (*vertex_count)) * face_mask.face_bit;

        quad_indices(i_masked, v_masked);
        plus_y_vertices(v_masked * face_mask.dir_bit, temp_uv ,x_f32 ,y_f32, z_f32);
        minus_y_vertices(v_masked * (1u ^ face_mask.dir_bit), temp_uv, x_f32, y_f32, z_f32);
        (*index_count) += (6u * face_mask.face_bit);
        (*vertex_count) += (4u * face_mask.face_bit);
    }
}

fn write_mesh_into_buffers(index_offset: u32, vertex_offset: u32) {
    var index_count: u32 = 0u;
    var vertex_count: u32 = 0u;
    for (var x: u32 = 0u; x < CHUNK_DIM_U16; x++) {
        for (var y: u32 = 0u; y < CHUNK_DIM_U16; y++) {
            for (var z: u32 = 0u; z < CHUNK_DIM_U32; z++) {
                write_faces_x(
                    index_offset,
                    &index_count,
                    vertex_offset,
                    &vertex_count,
                    x,
                    y,
                    z,
                );
                write_faces_y(
                    index_offset,
                    &index_count,
                    vertex_offset,
                    &vertex_count,
                    x,
                    y,
                    z,
                );
            }
        }
    }
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
