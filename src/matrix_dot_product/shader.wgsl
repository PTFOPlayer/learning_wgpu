@group(0)
@binding(0)
var<uniform> x: Matrix; 
@group(0)
@binding(1)
var<uniform> y: Matrix;
@group(0)
@binding(2)
var<storage, read_write> out: array<i32>;

fn dot_product(x_cord: u32, y_cord: u32) {

    var x_y = x.size_y;
    var y_x = x.size_x;

    var out_cord = x_cord * x_y + y_cord;


    var sum = 0;
    for (var i = u32(0); i < x_y; i += u32(1)) {
        sum += x.data[(x_cord * x_y) + i].x * y.data[(y_x * i) + y_cord].x;
    }
    out[out_cord] = sum;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    dot_product(global_id.x, global_id.y);
}

struct Matrix {
    data: array<vec4<i32>, 64>, 
    size_x: u32,
    size_y: u32,
}