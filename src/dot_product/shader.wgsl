@group(0)
@binding(0)
var<storage> x: array<i32>; 
@group(0)
@binding(1)
var<storage> y: array<i32>;
@group(0)
@binding(2)
var<storage, read_write> out: array<i32>;

fn dot_product(x_cord: u32, y_cord: u32) {
    var x_size = arrayLength(&x);
    var out_cord = x_cord*x_size+y_cord;
    out[out_cord] = x[x_cord]*y[y_cord];
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    dot_product(global_id.x, global_id.y);
}