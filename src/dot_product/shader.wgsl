@group(0)
@binding(0)
var<storage> x: array<i32>; 
@group(0)
@binding(1)
var<storage> y: array<i32>;
@group(0)
@binding(2)
var<storage, read_write> out: array<i32>;

@group(0)
@binding(3)
var<storage> sizes: array<u32>;

fn dot_product(x_cord: u32, y_cord: u32) {
    out[x_cord*sizes[0] + y_cord] = x[x_cord]*y[y_cord];
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    dot_product(global_id.x, global_id.y);
}