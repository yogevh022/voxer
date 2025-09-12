
fn neighbor_count_to_vao(count: u32) -> f32 {
    return 1.0 - (f32(count) * VAO_FACTOR);
}

// vao_* returns vao values on axis A for <A, +A>

fn vao_x(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<u32, 2> {
    var vao_front: u32;
    var vao_back: u32;

    let px_top = (*neighbors)[2][2][1];
    let px_bottom = (*neighbors)[2][0][1];
    let px_left = (*neighbors)[2][1][0];
    let px_right = (*neighbors)[2][1][2];
    let px_top_left = (1 ^ (px_top & px_left)) * (*neighbors)[2][2][0];
    let px_top_right = (1 ^ (px_top & px_right)) * (*neighbors)[2][2][2];
    let px_bottom_left = (1 ^ (px_bottom & px_left)) * (*neighbors)[2][0][0];
    let px_bottom_right = (1 ^ (px_bottom & px_right)) * (*neighbors)[2][0][2];

    vao_front = neighbor_count_to_vao(px_top + px_left + px_top_left)
        | (neighbor_count_to_vao(px_top + px_right + px_top_right) << 2)
        | (neighbor_count_to_vao(px_bottom + px_right + px_bottom_right) << 4)
        | (neighbor_count_to_vao(px_bottom + px_left + px_bottom_left) << 6);

    let sx_top = (*neighbors)[1][2][1];
    let sx_bottom = (*neighbors)[1][0][1];
    let sx_left = (*neighbors)[1][1][0];
    let sx_right = (*neighbors)[1][1][2];
    let sx_top_left = (1 ^ (sx_top & sx_left)) * (*neighbors)[1][2][0];
    let sx_top_right = (1 ^ (sx_top & sx_right)) * (*neighbors)[1][2][2];
    let sx_bottom_left = (1 ^ (sx_bottom & sx_left)) * (*neighbors)[1][0][0];
    let sx_bottom_right = (1 ^ (sx_bottom & sx_right)) * (*neighbors)[1][0][2];

    vao_back = neighbor_count_to_vao(sx_top + sx_left + sx_top_left)
        | (neighbor_count_to_vao(sx_top + sx_right + sx_top_right) << 2)
        | (neighbor_count_to_vao(sx_bottom + sx_right + sx_bottom_right) << 4)
        | (neighbor_count_to_vao(sx_bottom + sx_left + sx_bottom_left) << 6);

    return array<u32, 2>(vao_back, vao_front);
}

fn vao_y(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<array<f32, 4>, 2> {
    var vao_front: array<f32, 4>;
    var vao_back: array<f32, 4>;

    let py_top = (*neighbors)[2][2][1];
    let py_bottom = (*neighbors)[0][2][1];
    let py_left = (*neighbors)[1][2][0];
    let py_right = (*neighbors)[1][2][2];
    let py_top_left = (1 ^ (py_top & py_left)) * (*neighbors)[2][2][0];
    let py_top_right = (1 ^ (py_top & py_right)) * (*neighbors)[2][2][2];
    let py_bottom_left = (1 ^ (py_bottom & py_left)) * (*neighbors)[0][2][0];
    let py_bottom_right = (1 ^ (py_bottom & py_right)) * (*neighbors)[0][2][2];

    vao_front[0] = neighbor_count_to_vao(py_top + py_right + py_top_right);
    vao_front[1] = neighbor_count_to_vao(py_top + py_left + py_top_left);
    vao_front[2] = neighbor_count_to_vao(py_bottom + py_left + py_bottom_left);
    vao_front[3] = neighbor_count_to_vao(py_bottom + py_right + py_bottom_right);

    let sy_top = (*neighbors)[2][1][1];
    let sy_bottom = (*neighbors)[0][1][1];
    let sy_left = (*neighbors)[1][1][0];
    let sy_right = (*neighbors)[1][1][2];
    let sy_top_left = (1 ^ (sy_top & sy_left)) * (*neighbors)[2][1][0];
    let sy_top_right = (1 ^ (sy_top & sy_right)) * (*neighbors)[2][1][2];
    let sy_bottom_left = (1 ^ (sy_bottom & sy_left)) * (*neighbors)[0][1][0];
    let sy_bottom_right = (1 ^ (sy_bottom & sy_right)) * (*neighbors)[0][1][2];

    vao_back[0] = neighbor_count_to_vao(sy_top + sy_right + sy_top_right);
    vao_back[1] = neighbor_count_to_vao(sy_top + sy_left + sy_top_left);
    vao_back[2] = neighbor_count_to_vao(sy_bottom + sy_left + sy_bottom_left);
    vao_back[3] = neighbor_count_to_vao(sy_bottom + sy_right + sy_bottom_right);

    return array<array<f32, 4>, 2>(vao_back, vao_front);
}

fn vao_z(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<array<f32, 4>, 2> {
    var vao_front: array<f32, 4>;
    var vao_back: array<f32, 4>;

    let pz_top = (*neighbors)[1][2][2];
    let pz_bottom = (*neighbors)[1][0][2];
    let pz_left = (*neighbors)[0][1][2];
    let pz_right = (*neighbors)[2][1][2];
    let pz_top_left = (1 ^ (pz_top & pz_left)) * (*neighbors)[0][2][2];
    let pz_top_right = (1 ^ (pz_top & pz_right)) * (*neighbors)[2][2][2];
    let pz_bottom_left = (1 ^ (pz_bottom & pz_left)) * (*neighbors)[0][0][2];
    let pz_bottom_right = (1 ^ (pz_bottom & pz_right)) * (*neighbors)[2][0][2];

    vao_front[0] = neighbor_count_to_vao(pz_bottom + pz_left + pz_bottom_left);
    vao_front[1] = neighbor_count_to_vao(pz_bottom + pz_right + pz_bottom_right);
    vao_front[2] = neighbor_count_to_vao(pz_top + pz_right + pz_top_right);
    vao_front[3] = neighbor_count_to_vao(pz_top + pz_left + pz_top_left);

    let sz_top = (*neighbors)[1][2][1];
    let sz_bottom = (*neighbors)[1][0][1];
    let sz_left = (*neighbors)[0][1][1];
    let sz_right = (*neighbors)[2][1][1];
    let sz_top_left = (1 ^ (sz_top & sz_left)) * (*neighbors)[0][2][1];
    let sz_top_right = (1 ^ (sz_top & sz_right)) * (*neighbors)[2][2][1];
    let sz_bottom_left = (1 ^ (sz_bottom & sz_left)) * (*neighbors)[0][0][1];
    let sz_bottom_right = (1 ^ (sz_bottom & sz_right)) * (*neighbors)[2][0][1];

    vao_back[0] = neighbor_count_to_vao(sz_bottom + sz_left + sz_bottom_left);
    vao_back[1] = neighbor_count_to_vao(sz_bottom + sz_right + sz_bottom_right);
    vao_back[2] = neighbor_count_to_vao(sz_top + sz_right + sz_top_right);
    vao_back[3] = neighbor_count_to_vao(sz_top + sz_left + sz_top_left);

    return array<array<f32, 4>, 2>(vao_back, vao_front);
}