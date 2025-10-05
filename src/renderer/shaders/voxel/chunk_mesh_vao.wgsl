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

    let sx_top = (*neighbors)[1][2][1];
    let sx_bottom = (*neighbors)[1][0][1];
    let sx_left = (*neighbors)[1][1][0];
    let sx_right = (*neighbors)[1][1][2];

    let px_top_left = (*neighbors)[2][2][0];
    let px_top_right = (*neighbors)[2][2][2];
    let px_bottom_left = (*neighbors)[2][0][0];
    let px_bottom_right = (*neighbors)[2][0][2];

    let sx_top_left = (*neighbors)[1][2][0];
    let sx_top_right = (*neighbors)[1][2][2];
    let sx_bottom_left = (*neighbors)[1][0][0];
    let sx_bottom_right = (*neighbors)[1][0][2];

    let any_top = sx_top | px_top;
    let any_bottom = sx_bottom | px_bottom;
    let any_left = sx_left | px_left;
    let any_right = sx_right | px_right;

    let any_tl = sx_top_left | px_top_left;
    let any_tr = sx_top_right | px_top_right;
    let any_bl = sx_bottom_left | px_bottom_left;
    let any_br = sx_bottom_right | px_bottom_right;

    let px_bleft_ao = px_left & (any_bottom | any_bl);
    let px_bright_ao = px_right & (any_bottom | any_br);
    let px_tleft_ao = px_left & (any_top | any_tl);
    let px_tright_ao = px_right & (any_top | any_tr);

    let px_topr_ao = px_top & (any_right | any_tr);
    let px_topl_ao = px_top & (any_left | any_tl);
    let px_bottomr_ao = px_bottom & (any_right | any_br);
    let px_bottoml_ao = px_bottom & (any_left | any_bl);

    let sx_bleft_ao = sx_left & (any_bottom | any_bl);
    let sx_bright_ao = sx_right & (any_bottom | any_br);
    let sx_tleft_ao = sx_left & (any_top | any_tl);
    let sx_tright_ao = sx_right & (any_top | any_tr);

    let sx_topr_ao = sx_top & (any_right | any_tr);
    let sx_topl_ao = sx_top & (any_left | any_tl);
    let sx_bottomr_ao = sx_bottom & (any_right | any_br);
    let sx_bottoml_ao = sx_bottom & (any_left | any_bl);

    vao_front = max(px_topl_ao + px_tleft_ao, px_top_left)
        | (max(px_topr_ao + px_tright_ao, px_top_right) << 2)
        | (max(px_bottomr_ao + px_bright_ao, px_bottom_right) << 4)
        | (max(px_bottoml_ao + px_bleft_ao, px_bottom_left) << 6);

    vao_back = max(sx_topr_ao + sx_tright_ao, sx_top_right)
        | (max(sx_topl_ao + sx_tleft_ao, sx_top_left) << 2)
        | (max(sx_bottoml_ao + sx_bleft_ao, sx_bottom_left) << 4)
        | (max(sx_bottomr_ao + sx_bright_ao, sx_bottom_right) << 6);

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

    vao_front = max(py_topr_ao + py_tright_ao, py_top_right)
        | (max(py_topl_ao + py_tleft_ao, py_top_left) << 2)
        | (max(py_bottoml_ao + py_bleft_ao, py_bottom_left) << 4)
        | (max(py_bottomr_ao + py_bright_ao, py_bottom_right) << 6);

    vao_back = max(sy_topl_ao + sy_tleft_ao, sy_top_left)
        | (max(sy_topr_ao + sy_tright_ao, sy_top_right) << 2)
        | (max(sy_bottomr_ao + sy_bright_ao, sy_bottom_right) << 4)
        | (max(sy_bottoml_ao + sy_bleft_ao, sy_bottom_left) << 6);

    return array<u32, 2>(vao_back, vao_front);
}

fn occlusion_count_z(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<u32, 2> {
    var vao_front: u32;
    var vao_back: u32;

    let pz_top = (*neighbors)[1][2][2];
    let pz_bottom = (*neighbors)[1][0][2];
    let pz_left = (*neighbors)[0][1][2];
    let pz_right = (*neighbors)[2][1][2];

    let sz_top = (*neighbors)[1][2][1];
    let sz_bottom = (*neighbors)[1][0][1];
    let sz_left = (*neighbors)[0][1][1];
    let sz_right = (*neighbors)[2][1][1];

    let pz_top_left = (*neighbors)[0][2][2];
    let pz_top_right = (*neighbors)[2][2][2];
    let pz_bottom_left = (*neighbors)[0][0][2];
    let pz_bottom_right = (*neighbors)[2][0][2];

    let sz_top_left = (*neighbors)[0][2][1];
    let sz_top_right = (*neighbors)[2][2][1];
    let sz_bottom_left = (*neighbors)[0][0][1];
    let sz_bottom_right = (*neighbors)[2][0][1];

    let any_top = sz_top | pz_top;
    let any_bottom = sz_bottom | pz_bottom;
    let any_left = sz_left | pz_left;
    let any_right = sz_right | pz_right;

    let any_tl = sz_top_left | pz_top_left;
    let any_tr = sz_top_right | pz_top_right;
    let any_bl = sz_bottom_left | pz_bottom_left;
    let any_br = sz_bottom_right | pz_bottom_right;

    let pz_bleft_ao = pz_left & (any_bottom | any_bl);
    let pz_bright_ao = pz_right & (any_bottom | any_br);
    let pz_tleft_ao = pz_left & (any_top | any_tl);
    let pz_tright_ao = pz_right & (any_top | any_tr);

    let pz_topr_ao = pz_top & (any_right | any_tr);
    let pz_topl_ao = pz_top & (any_left | any_tl);
    let pz_bottomr_ao = pz_bottom & (any_right | any_br);
    let pz_bottoml_ao = pz_bottom & (any_left | any_bl);

    let sz_bleft_ao = sz_left & (any_bottom | any_bl);
    let sz_bright_ao = sz_right & (any_bottom | any_br);
    let sz_tleft_ao = sz_left & (any_top | any_tl);
    let sz_tright_ao = sz_right & (any_top | any_tr);

    let sz_topr_ao = sz_top & (any_right | any_tr);
    let sz_topl_ao = sz_top & (any_left | any_tl);
    let sz_bottomr_ao = sz_bottom & (any_right | any_br);
    let sz_bottoml_ao = sz_bottom & (any_left | any_bl);

    vao_front = max(pz_bottoml_ao + pz_bleft_ao, pz_bottom_left)
        | (max(pz_bottomr_ao + pz_bright_ao, pz_bottom_right) << 2)
        | (max(pz_topr_ao + pz_tright_ao, pz_top_right) << 4)
        | (max(pz_topl_ao + pz_tleft_ao, pz_top_left) << 6);

    vao_back = max(sz_bottomr_ao + sz_bright_ao, sz_bottom_right)
        | (max(sz_bottoml_ao + sz_bleft_ao, sz_bottom_left) << 2)
        | (max(sz_topl_ao + sz_tleft_ao, sz_top_left) << 4)
        | (max(sz_topr_ao + sz_tright_ao, sz_top_right) << 6);

    return array<u32, 2>(vao_back, vao_front);
}
