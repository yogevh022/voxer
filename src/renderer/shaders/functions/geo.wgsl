fn isquare_distance(a: vec3<i32>, b: vec3<i32>) -> i32 {
    let diff = a - b;
    return dot(diff, diff);
}

fn square_distance(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let diff = a - b;
    return dot(diff, diff);
}