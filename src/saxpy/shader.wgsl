@group(0)
@binding(0)
var<storage, read_write> x: array<i32>; 
@group(0)
@binding(1)
var<storage> y: array<i32>;
@group(0)
@binding(2)
var<storage> a: array<i32>;

fn saxpy(x: i32, y: i32) -> i32 {
    var result = (a[0]*x) + y;
    return result;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    x[global_id.x] = saxpy(x[global_id.x], y[global_id.x]);
}