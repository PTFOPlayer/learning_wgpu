@group(0)
@binding(0)
var<storage> x: array<i32>; 
@group(0)
@binding(1)
var<storage, read_write> out: array<i32>;

fn transpose(x_cord: u32, y_cord: u32) {
    out[y_cord * 4 + x_cord] = x[x_cord * 4 + y_cord];
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    transpose(global_id.x, global_id.y);
}