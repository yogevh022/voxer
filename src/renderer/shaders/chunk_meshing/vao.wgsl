
fn neighbor_count_to_vao(count: u32) -> f32 {
    return 1.0 - (f32(count) * VAO_FACTOR);
}

// vao_* returns vao values on axis A for <-A, +A>

fn vao_x(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<array<f32, 4>, 2> {
    var vao_front: array<f32, 4>;
    var vao_back: array<f32, 4>;

    let px_top = (*neighbors)[2][2][1];
    let px_bottom = (*neighbors)[2][0][1];
    let px_left = (*neighbors)[2][1][2];
    let px_right = (*neighbors)[2][1][0];
    let px_top_left = (1 ^ (px_top & px_left)) * (*neighbors)[2][2][2];
    let px_top_right = (1 ^ (px_top & px_right)) * (*neighbors)[2][2][0];
    let px_bottom_left = (1 ^ (px_bottom & px_left)) * (*neighbors)[2][0][2];
    let px_bottom_right = (1 ^ (px_bottom & px_right)) * (*neighbors)[2][0][0];

    vao_front[2] = neighbor_count_to_vao(px_top + px_left + px_top_left);
    vao_front[3] = neighbor_count_to_vao(px_top + px_right + px_top_right);
    vao_front[1] = neighbor_count_to_vao(px_bottom + px_left + px_bottom_left);
    vao_front[0] = neighbor_count_to_vao(px_bottom + px_right + px_bottom_right);

    let sx_top = (*neighbors)[1][2][1];
    let sx_bottom = (*neighbors)[1][0][1];
    let sx_left = (*neighbors)[1][1][2];
    let sx_right = (*neighbors)[1][1][0];
    let sx_top_left = (1 ^ (sx_top & sx_left)) * (*neighbors)[1][2][2];
    let sx_top_right = (1 ^ (sx_top & sx_right)) * (*neighbors)[1][2][0];
    let sx_bottom_left = (1 ^ (sx_bottom & sx_left)) * (*neighbors)[1][0][2];
    let sx_bottom_right = (1 ^ (sx_bottom & sx_right)) * (*neighbors)[1][0][0];

    vao_back[2] = neighbor_count_to_vao(sx_top + sx_left + sx_top_left);
    vao_back[3] = neighbor_count_to_vao(sx_top + sx_right + sx_top_right);
    vao_back[1] = neighbor_count_to_vao(sx_bottom + sx_left + sx_bottom_left);
    vao_back[0] = neighbor_count_to_vao(sx_bottom + sx_right + sx_bottom_right);

    return array<array<f32, 4>, 2>(vao_back, vao_front);
}

fn vao_y(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<array<f32, 4>, 2> {
    var vao_front: array<f32, 4>;
    var vao_back: array<f32, 4>;

    // fixme shared edge is +x: top -> +y: right

    let py_top = (*neighbors)[2][2][1];
    let py_bottom = (*neighbors)[0][2][1];
    let py_left = (*neighbors)[1][2][2];
    let py_right = (*neighbors)[1][2][0];
    let py_top_left = (1 ^ (py_top & py_left)) * (*neighbors)[2][2][2];
    let py_top_right = (1 ^ (py_top & py_right)) * (*neighbors)[2][2][0];
    let py_bottom_left = (1 ^ (py_bottom & py_left)) * (*neighbors)[0][2][2];
    let py_bottom_right = (1 ^ (py_bottom & py_right)) * (*neighbors)[0][2][0];

    vao_front[3] = neighbor_count_to_vao(py_top + py_right);
    vao_front[2] = neighbor_count_to_vao(py_top + py_left + py_top_left);
    vao_front[1] = neighbor_count_to_vao(py_bottom + py_left + py_bottom_left);
    vao_front[0] = neighbor_count_to_vao(py_bottom + py_right);

    let sy_top = (*neighbors)[2][1][1];
    let sy_bottom = (*neighbors)[0][1][1];
    let sy_left = (*neighbors)[1][1][2];
    let sy_right = (*neighbors)[1][1][0];
    let sy_top_left = (1 ^ (sy_top & sy_left)) * (*neighbors)[2][1][2];
    let sy_top_right = (1 ^ (sy_top & sy_right)) * (*neighbors)[2][1][0];
    let sy_bottom_left = (1 ^ (sy_bottom & sy_left)) * (*neighbors)[0][1][2];
    let sy_bottom_right = (1 ^ (sy_bottom & sy_right)) * (*neighbors)[0][1][0];

    vao_back[2] = neighbor_count_to_vao(sy_top + sy_left + sy_top_left);
    vao_back[3] = neighbor_count_to_vao(sy_top + sy_right + sy_top_right);
    vao_back[1] = neighbor_count_to_vao(sy_bottom + sy_left + sy_bottom_left);
    vao_back[0] = neighbor_count_to_vao(sy_bottom + sy_right + sy_bottom_right);

    return array<array<f32, 4>, 2>(vao_back, vao_front);
}

fn vao_z(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<array<f32, 4>, 2> {
    var vao_front: array<f32, 4>;
    var vao_back: array<f32, 4>;

    let _122 = (*neighbors)[1][2][2];
    let _102 = (*neighbors)[1][0][2];
    let _212 = (*neighbors)[2][1][2];
    let _012 = (*neighbors)[0][1][2];

    vao_front[2] = neighbor_count_to_vao(_212 + _122);
    vao_front[3] = neighbor_count_to_vao(_122 + _012);
    vao_front[0] = neighbor_count_to_vao(_012 + _102);
    vao_front[1] = neighbor_count_to_vao(_102 + _212);

    let _121 = (*neighbors)[1][2][1];
    let _101 = (*neighbors)[1][0][1];
    let _211 = (*neighbors)[2][1][1];
    let _011 = (*neighbors)[0][1][1];

    vao_back[2] = neighbor_count_to_vao(_211 + _121);
    vao_back[3] = neighbor_count_to_vao(_121 + _011);
    vao_back[0] = neighbor_count_to_vao(_011 + _101);
    vao_back[1] = neighbor_count_to_vao(_101 + _211);

    return array<array<f32, 4>, 2>(vao_back, vao_front);
}