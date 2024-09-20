
struct VertexOut {
  @location(0) color: vec4<f32>,
  @builtin(position) position: vec4<f32>
}

var<private>arr: array<vec4<f32>,3> = array(
  vec4<f32>(1.0, 0.0, 0.0, 1.0),
  vec4<f32>(0.0, 1.0, 0.0, 1.0),
  vec4<f32>(0.0, 0.0, 1.0, 1.0)
);


@vertex
fn vertex_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOut {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    
    var out: VertexOut;
    
    out.color = arr[in_vertex_index];
    out.position =vec4<f32>(x, y, 0.0, 1.0);

    return out;
}

@fragment
fn fragment_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
