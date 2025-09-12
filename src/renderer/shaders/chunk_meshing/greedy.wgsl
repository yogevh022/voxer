struct FaceData {
    voxel_type__ao__illum__fid: u32,
    // voxel_type: 16b
    // ao: 8b (2b * 4)
    // illum: 5b
    // fid: 3b
    pos_a__pos_b: u32
    // pos_a: 12b
    // pos_b: 12b
    // 8b free
}

fn expand() {
    if (i + 1 == CHUNK_DIM) {
        // update new greedy quad
        return;
    }
    // todo optimize visited pattern
    for (var ei = i + 1; ei < CHUNK_DIM; ei++) {
        for (var ej = last_voxel_j; ej < j; ej++) {
            if (slice[ei][ej] == last_voxel) { // && block on +-adjacent 3rd axis is air && ao is the same as last_voxel
                // set visited
            } else {
                for (var uej = last_voxel_j; uej < ej; uej++) {
                    // unset visited
                }
                return;
            }
        }
        // update new greedy quad
    }
}

fn mesh() {
    var slice: array<array<u32, CHUNK_DIM_HALF>, CHUNK_DIM>;
    var visited: array<u32, CHUNK_DIM_HALF>;

    var current_point = 0;
    var last_voxel = slice[0][0];
    var last_voxel_j = 0;
    // calc + set initial last ao
    for (var i = 0; i < CHUNK_DIM; i++) {
        for (var j = 0; j < CHUNK_DIM; j++) {
            let current_voxel = slice[i][j];
            if (current_voxel == 0) { // || visited current_voxel
                continue;
            }
            // calc ao
            // set self visited
            if (current_voxel != last_voxel) { // current ao != last_ao
                expand();
                last_voxel = current_voxel;
                last_voxel_j = j;
                // set last ao
            }
        }
    }
    // final expand logic
}