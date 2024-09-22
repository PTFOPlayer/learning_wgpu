
struct VertexIn {
  @location(0) position: vec4<f32>,
  @location(1) color: vec4<f32>
}

struct VertexOut {
  @location(0) color: vec4<f32>,
  @builtin(position) position: vec4<f32>
}

@vertex
fn vertex_main(vertex_in: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.color = vertex_in.color;
    out.position = vertex_in.position;

    return out;
}

@fragment
fn fragment_main(v_in: VertexOut) -> @location(0) vec4<f32> {
    return v_in.color;
}
