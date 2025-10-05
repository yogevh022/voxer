// occlusion_count_* returns occlusion count on axis A for <A, +A>

fn occlusion_count_to_ao(ocl_count: u32) -> f32 {
    return 1.0 - (f32(ocl_count) * CFG_VAO_FACTOR);
}

fn occlusion_count_x(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<u32, 2> {
    var vao_front: u32;
    var vao_back: u32;

    let px_top = (*neighbors)[2][2][1];
    let px_bottom = (*neighbors)[2][0][1];
    let px_left = (*neighbors)[2][1][0];
    let px_right = (*neighbors)[2][1][2];
    let px_top_left = (1 ^ (px_top | px_left)) * (*neighbors)[2][2][0];
    let px_top_right = (1 ^ (px_top | px_right)) * (*neighbors)[2][2][2];
    let px_bottom_left = (1 ^ (px_bottom | px_left)) * (*neighbors)[2][0][0];
    let px_bottom_right = (1 ^ (px_bottom | px_right)) * (*neighbors)[2][0][2];

    vao_front = (px_top + px_left + px_top_left)
        | ((px_top + px_right + px_top_right) << 2)
        | ((px_bottom + px_right + px_bottom_right) << 4)
        | ((px_bottom + px_left + px_bottom_left) << 6);

    let sx_top = (*neighbors)[1][2][1];
    let sx_bottom = (*neighbors)[1][0][1];
    let sx_left = (*neighbors)[1][1][0];
    let sx_right = (*neighbors)[1][1][2];
    let sx_top_left = (1 ^ (sx_top | sx_left)) * (*neighbors)[1][2][0];
    let sx_top_right = (1 ^ (sx_top | sx_right)) * (*neighbors)[1][2][2];
    let sx_bottom_left = (1 ^ (sx_bottom | sx_left)) * (*neighbors)[1][0][0];
    let sx_bottom_right = (1 ^ (sx_bottom | sx_right)) * (*neighbors)[1][0][2];

    vao_back = (sx_top + sx_right + sx_top_right)
        | ((sx_top + sx_left + sx_top_left) << 2)
        | ((sx_bottom + sx_left + sx_bottom_left) << 4)
        | ((sx_bottom + sx_right + sx_bottom_right) << 6);

    return array<u32, 2>(vao_back, vao_front);
}

fn occlusion_count_y(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<u32, 2> {
    var vao_front: u32;
    var vao_back: u32;

    let py_top = (*neighbors)[2][2][1];
    let py_bottom = (*neighbors)[0][2][1];
    let py_left = (*neighbors)[1][2][0];
    let py_right = (*neighbors)[1][2][2];

    let sy_top = (*neighbors)[2][1][1];
    let sy_bottom = (*neighbors)[0][1][1];
    let sy_left = (*neighbors)[1][1][0];
    let sy_right = (*neighbors)[1][1][2];

    let py_top_left = (*neighbors)[2][2][0];
    let py_top_right = (*neighbors)[2][2][2];
    let py_bottom_left = (*neighbors)[0][2][0];
    let py_bottom_right = (*neighbors)[0][2][2];

    let sy_top_left = (*neighbors)[2][1][0];
    let sy_top_right = (*neighbors)[2][1][2];
    let sy_bottom_left = (*neighbors)[0][1][0];
    let sy_bottom_right = (*neighbors)[0][1][2];

    let any_top = sy_top | py_top;
    let any_bottom = sy_bottom | py_bottom;
    let any_left = sy_left | py_left;
    let any_right = sy_right | py_right;

    let any_tl = sy_top_left | py_top_left;
    let any_tr = sy_top_right | py_top_right;
    let any_bl = sy_bottom_left | py_bottom_left;
    let any_br = sy_bottom_right | py_bottom_right;

    let py_bleft_ao = py_left & (any_bottom | any_bl);
    let py_bright_ao = py_right & (any_bottom | any_br);
    let py_tleft_ao = py_left & (any_top | any_tl);
    let py_tright_ao = py_right & (any_top | any_tr);

    let py_topr_ao = py_top & (any_right | any_tr);
    let py_topl_ao = py_top & (any_left | any_tl);
    let py_bottomr_ao = py_bottom & (any_right | any_br);
    let py_bottoml_ao = py_bottom & (any_left | any_bl);

    let sy_bleft_ao = sy_left & (any_bottom | any_bl);
    let sy_bright_ao = sy_right & (any_bottom | any_br);
    let sy_tleft_ao = sy_left & (any_top | any_tl);
    let sy_tright_ao = sy_right & (any_top | any_tr);

    let sy_topr_ao = sy_top & (any_right | any_tr);
    let sy_topl_ao = sy_top & (any_left | any_tl);
    let sy_bottomr_ao = sy_bottom & (any_right | any_br);
    let sy_bottoml_ao = sy_bottom & (any_left | any_bl);

    vao_front = max(py_tright_ao + py_topr_ao, py_top_right)
        | (max(py_tleft_ao + py_topl_ao, py_top_left) << 2)
        | (max(py_bleft_ao + py_bottoml_ao, py_bottom_left) << 4)
        | (max(py_bright_ao + py_bottomr_ao, py_bottom_right) << 6);

    vao_back = max(sy_tleft_ao + sy_topl_ao, sy_top_left)
        | (max(sy_tright_ao + sy_topr_ao, sy_top_right) << 2)
        | (max(sy_bright_ao + sy_bottomr_ao, sy_bottom_right) << 4)
        | (max(sy_bleft_ao + sy_bottoml_ao, sy_bottom_left) << 6);

    return array<u32, 2>(vao_back, vao_front);
}

fn occlusion_count_z(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<u32, 2> {
    var vao_front: u32;
    var vao_back: u32;

    let pz_top = (*neighbors)[1][2][2];
    let pz_bottom = (*neighbors)[1][0][2];
    let pz_left = (*neighbors)[0][1][2];
    let pz_right = (*neighbors)[2][1][2];
    let pz_top_left = (1 ^ (pz_top | pz_left)) * (*neighbors)[0][2][2];
    let pz_top_right = (1 ^ (pz_top | pz_right)) * (*neighbors)[2][2][2];
    let pz_bottom_left = (1 ^ (pz_bottom | pz_left)) * (*neighbors)[0][0][2];
    let pz_bottom_right = (1 ^ (pz_bottom | pz_right)) * (*neighbors)[2][0][2];

    vao_front = (pz_bottom + pz_left + pz_bottom_left)
        | ((pz_bottom + pz_right + pz_bottom_right) << 2)
        | ((pz_top + pz_right + pz_top_right) << 4)
        | ((pz_top + pz_left + pz_top_left) << 6);

    let sz_top = (*neighbors)[1][2][1];
    let sz_bottom = (*neighbors)[1][0][1];
    let sz_left = (*neighbors)[0][1][1];
    let sz_right = (*neighbors)[2][1][1];
    let sz_top_left = (1 ^ (sz_top | sz_left)) * (*neighbors)[0][2][1];
    let sz_top_right = (1 ^ (sz_top | sz_right)) * (*neighbors)[2][2][1];
    let sz_bottom_left = (1 ^ (sz_bottom | sz_left)) * (*neighbors)[0][0][1];
    let sz_bottom_right = (1 ^ (sz_bottom | sz_right)) * (*neighbors)[2][0][1];

    vao_back = (sz_bottom + sz_right + sz_bottom_right)
        | ((sz_bottom + sz_left + sz_bottom_left) << 2)
        | ((sz_top + sz_left + sz_top_left) << 4)
        | ((sz_top + sz_right + sz_top_right) << 6);

    return array<u32, 2>(vao_back, vao_front);
}
