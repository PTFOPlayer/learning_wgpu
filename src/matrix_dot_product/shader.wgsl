@group(0)
@binding(0)
var<storage> x: array<i32>; 
@group(0)
@binding(1)
var<storage> y: array<i32>;
@group(0)
@binding(2)
var<storage> n_buff: array<u32>;
@group(0)
@binding(3)
var<storage, read_write> out: array<i32>;

fn dot_product(x_cord: u32, y_cord: u32) {
    let n = n_buff[0];

    var out_cord = x_cord * n + y_cord;

    var sum = 0;
    for (var i = u32(0); i < n; i += u32(1)) {
        sum += x[(x_cord * n) + i] * y[(n * i) + y_cord];
    }

    out[out_cord] = sum;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    dot_product(global_id.x, global_id.y);
}