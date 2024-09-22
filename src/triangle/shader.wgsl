
struct VertexOut {
  @location(0) color: vec4<f32>,
  @builtin(position) position: vec4<f32>
}

var<private>positions: array<vec4<f32>, 3> = array(
  vec4<f32>(0.0, 0.5, 0.0, 1.0),
  vec4<f32>(-0.5, -0.5, 0.0, 1.0),
  vec4<f32>(0.5, -0.5, 0.0, 1.0)
);

var<private>colors: array<vec4<f32>,3> = array(
  vec4<f32>(1.0, 0.0, 0.0, 1.0),
  vec4<f32>(0.0, 1.0, 0.0, 1.0),
  vec4<f32>(0.0, 0.0, 1.0, 1.0)
);


@vertex
fn vertex_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOut {

    var out: VertexOut;
    
    out.color = colors[in_vertex_index];
    out.position = positions[in_vertex_index];

    return out;
}

@fragment
fn fragment_main(v_in: VertexOut) -> @location(0) vec4<f32> {
    return v_in.color;
}
