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
    let px_top_left = (1 ^ (px_top & px_left)) * (*neighbors)[2][2][0];
    let px_top_right = (1 ^ (px_top & px_right)) * (*neighbors)[2][2][2];
    let px_bottom_left = (1 ^ (px_bottom & px_left)) * (*neighbors)[2][0][0];
    let px_bottom_right = (1 ^ (px_bottom & px_right)) * (*neighbors)[2][0][2];

    vao_front = (px_top + px_left + px_top_left)
        | ((px_top + px_right + px_top_right) << 2)
        | ((px_bottom + px_right + px_bottom_right) << 4)
        | ((px_bottom + px_left + px_bottom_left) << 6);

    let sx_top = (*neighbors)[1][2][1];
    let sx_bottom = (*neighbors)[1][0][1];
    let sx_left = (*neighbors)[1][1][0];
    let sx_right = (*neighbors)[1][1][2];
    let sx_top_left = (1 ^ (sx_top & sx_left)) * (*neighbors)[1][2][0];
    let sx_top_right = (1 ^ (sx_top & sx_right)) * (*neighbors)[1][2][2];
    let sx_bottom_left = (1 ^ (sx_bottom & sx_left)) * (*neighbors)[1][0][0];
    let sx_bottom_right = (1 ^ (sx_bottom & sx_right)) * (*neighbors)[1][0][2];

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
    let py_top_left = (1 ^ (py_top & py_left)) * (*neighbors)[2][2][0];
    let py_top_right = (1 ^ (py_top & py_right)) * (*neighbors)[2][2][2];
    let py_bottom_left = (1 ^ (py_bottom & py_left)) * (*neighbors)[0][2][0];
    let py_bottom_right = (1 ^ (py_bottom & py_right)) * (*neighbors)[0][2][2];

    vao_front = (py_top + py_right + py_top_right)
        | ((py_top + py_left + py_top_left) << 2)
        | ((py_bottom + py_left + py_bottom_left) << 4)
        | ((py_bottom + py_right + py_bottom_right) << 6);

    let sy_top = (*neighbors)[2][1][1];
    let sy_bottom = (*neighbors)[0][1][1];
    let sy_left = (*neighbors)[1][1][0];
    let sy_right = (*neighbors)[1][1][2];
    let sy_top_left = (1 ^ (sy_top & sy_left)) * (*neighbors)[2][1][0];
    let sy_top_right = (1 ^ (sy_top & sy_right)) * (*neighbors)[2][1][2];
    let sy_bottom_left = (1 ^ (sy_bottom & sy_left)) * (*neighbors)[0][1][0];
    let sy_bottom_right = (1 ^ (sy_bottom & sy_right)) * (*neighbors)[0][1][2];

    vao_back = (sy_top + sy_left + sy_top_left)
        | ((sy_top + sy_right + sy_top_right) << 2)
        | ((sy_bottom + sy_right + sy_bottom_right) << 4)
        | ((sy_bottom + sy_left + sy_bottom_left) << 6);

    return array<u32, 2>(vao_back, vao_front);
}

fn occlusion_count_z(neighbors: ptr<function, array<array<array<u32, 3>, 3>, 3>>) -> array<u32, 2> {
    var vao_front: u32;
    var vao_back: u32;

    let pz_top = (*neighbors)[1][2][2];
    let pz_bottom = (*neighbors)[1][0][2];
    let pz_left = (*neighbors)[0][1][2];
    let pz_right = (*neighbors)[2][1][2];
    let pz_top_left = (1 ^ (pz_top & pz_left)) * (*neighbors)[0][2][2];
    let pz_top_right = (1 ^ (pz_top & pz_right)) * (*neighbors)[2][2][2];
    let pz_bottom_left = (1 ^ (pz_bottom & pz_left)) * (*neighbors)[0][0][2];
    let pz_bottom_right = (1 ^ (pz_bottom & pz_right)) * (*neighbors)[2][0][2];

    vao_front = (pz_bottom + pz_left + pz_bottom_left)
        | ((pz_bottom + pz_right + pz_bottom_right) << 2)
        | ((pz_top + pz_right + pz_top_right) << 4)
        | ((pz_top + pz_left + pz_top_left) << 6);

    let sz_top = (*neighbors)[1][2][1];
    let sz_bottom = (*neighbors)[1][0][1];
    let sz_left = (*neighbors)[0][1][1];
    let sz_right = (*neighbors)[2][1][1];
    let sz_top_left = (1 ^ (sz_top & sz_left)) * (*neighbors)[0][2][1];
    let sz_top_right = (1 ^ (sz_top & sz_right)) * (*neighbors)[2][2][1];
    let sz_bottom_left = (1 ^ (sz_bottom & sz_left)) * (*neighbors)[0][0][1];
    let sz_bottom_right = (1 ^ (sz_bottom & sz_right)) * (*neighbors)[2][0][1];

    vao_back = (sz_bottom + sz_right + sz_bottom_right)
        | ((sz_bottom + sz_left + sz_bottom_left) << 2)
        | ((sz_top + sz_left + sz_top_left) << 4)
        | ((sz_top + sz_right + sz_top_right) << 6);

    return array<u32, 2>(vao_back, vao_front);
}
