use std::{default, fmt::Display};

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferUsages, Device, Queue};

use crate::{
    helpers::{create_bind_group, create_pipeline, create_staging_buffer, create_storage_buffer},
    Error,
};

// executes shader with given parameters
async fn execute_shader(
    matrix_x: Matrix,
    matrix_y: Matrix,
    device: &Device,
    queue: &Queue,
) -> Result<Vec<i32>, Error> {
    let out = vec![0; (matrix_x.y * matrix_y.x) as usize];
    let out_slice = out.as_slice();
    let size = size_of_val(out_slice) as wgpu::BufferAddress;
    // return buffer
    // MAP_READ allows for reading it
    // COPY_DST allows for it to be desetination of cpy
    let staging_buffer_out = create_staging_buffer(device, size);

    // output buffer that is avaliable for GPU
    let storage_buffer_out = create_storage_buffer(
        device,
        out_slice,
        BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
    );

    // buffer that is avaliable for GPU
    let storage_buffer_x = create_storage_buffer(device, &[matrix_x], BufferUsages::UNIFORM);

    // buffer that is avaliable for GPU
    let storage_buffer_y = create_storage_buffer(device, &[matrix_y], BufferUsages::UNIFORM);

    // creation of compute pipeline with entrypoint "main"
    let compute_pipeline = create_pipeline(device, include_str!("shader.wgsl"), "main");

    // binding buffer to group zero with specific bindings
    let bind_group = create_bind_group(
        device,
        &compute_pipeline,
        [
            (0, storage_buffer_x.as_entire_binding()),
            (1, storage_buffer_y.as_entire_binding()),
            (2, storage_buffer_out.as_entire_binding()),
        ],
    );

    // creates command encoder
    // its role is to execute pipelines (one ore more)
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // compute pass is invoked in other scope,
    // it needs to be dealocated before we can use encoder again
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("dot product");
        cpass.dispatch_workgroups(matrix_x.y, matrix_y.x, 1);
    }

    // copy result
    encoder.copy_buffer_to_buffer(&storage_buffer_out, 0, &staging_buffer_out, 0, size);

    queue.submit(Some(encoder.finish()));

    // sending data back to host
    let buffer_slice = staging_buffer_out.slice(..);

    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // await for GPU processes
    device.poll(wgpu::Maintain::wait()).panic_on_timeout();

    // await for buffer
    if let Ok(Ok(())) = receiver.recv_async().await {
        let data = buffer_slice.get_mapped_range();
        let result: Vec<i32> = bytemuck::cast_slice(&data).to_vec();
        // all veiws have to be dropped manualy
        drop(data);
        staging_buffer_out.unmap();

        return Ok(result);
    }

    Err(Error::ExecutionError)
}

pub async fn execute_matrix_dot_product(device: Device, queue: Queue) -> Result<(), Error> {
    #[rustfmt::skip]
    let x = [ 
        2, 1, 1, 7, 4, 3,
        1, 4, 8, 0, 3, 5,
        4, 1, 0, 5, 2, 1,
        4, 5, 7, 2, 6, 8,
        1, 6, 0, 9, 9, 7,
        3, 5, 2, 1, 4, 7,
 
    ];
    #[rustfmt::skip]
    let y = [
        5, 6, 9, 7, 1, 0,
        1, 6, 6, 0, 8, 3,
        1, 7, 2, 7, 1, 4,
        7, 1, 9, 5, 1, 2,
        0, 5, 1, 9, 5, 2,
        1, 8, 8, 6, 0, 6,
    ];

    let matrix_x = Matrix::new(&x, 6, 6);
    let matrix_y = Matrix::new(&y, 6, 6);

    let result = execute_shader(matrix_x, matrix_y, &device, &queue).await?;

    println!("x: {}", matrix_x);
    println!("y: {}", matrix_y);

    println!("result : [");
    for i in 0..6 {
        let idx = (i * 6) as usize;
        println!("  {:?}", &result[idx..idx + 6]);
    }
    println!("]");
    Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Matrix {
    data: [[i32; 4]; 8 * 8],
    x: u32,
    y: u32,
    _pad: [u32; 2],
}
impl Default for Matrix {
    fn default() -> Self {
        Self {
            data: [[0; 4]; 8 * 8],
            x: Default::default(),
            y: Default::default(),
            _pad: Default::default(),
        }
    }
}

impl Matrix {
    fn new(data: &[i32], x: u32, y: u32) -> Matrix {
        let mut out = Matrix {
            x,
            y,
            ..Default::default()
        };
        out.data[..data.len()]
            .copy_from_slice(&data.iter().map(|x| [*x, 0, 0, 0]).collect::<Vec<_>>());
        out
    }
}

impl Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut data = String::new();
        data += "[\n";

        let (dimx, dimy) = (self.x as usize, self.y as usize);

        for i in 0..dimx {
            for j in 0..dimy {
                data += &format!(" {},", self.data[i * dimx + j][0]);
            }
            data += "\n";
        }
        data += "]";

        f.write_str(&data)
    }
}
