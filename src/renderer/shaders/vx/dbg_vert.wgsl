struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn dbg_vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var via = array<u32, 6>(0, 2, 1, 0, 3, 2);

    var pos = array<vec2<f32>, 4>(
      vec2<f32>(-1, 1),
      vec2<f32>(0, 1),
      vec2<f32>(0, 0),
      vec2<f32>(-1, 0),
    );

    var uv = array<vec2<f32>, 4>(
      vec2<f32>(0.0, 0.0),
      vec2<f32>(0.5, 0.0),
      vec2<f32>(0.5, 0.5),
      vec2<f32>(0.0, 0.5),
    );

    let vi = via[i];
    let out_pos = pos[vi];
    let out_uv = uv[vi];

    var output: VertexOutput;
    output.pos = vec4<f32>(out_pos, 0.0, 1.0);
    output.uv = out_uv;
    return output;
}