use wgpu::{BufferUsages, Device, Queue};

use crate::{
    helpers::{create_bind_group, create_pipeline, create_staging_buffer, create_storage_buffer},
    Error,
};

// executes shader with given parameters
async fn execute_shader(
    x: &[i32],
    y: &[i32],
    n: u32,
    device: &Device,
    queue: &Queue,
) -> Result<Vec<i32>, Error> {
    let out = vec![0; (n * n) as usize];
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
    let storage_buffer_x = create_storage_buffer(device, x, BufferUsages::STORAGE);

    // buffer that is avaliable for GPU
    let storage_buffer_y = create_storage_buffer(device, y, BufferUsages::STORAGE);

    // buffer that is avaliable for GPU
    let storage_buffer_sizes = create_storage_buffer(device, &[n], BufferUsages::STORAGE);

    // creation of compute pipeline with entrypoint "main"
    let compute_pipeline = create_pipeline(device, include_str!("shader.wgsl"), "main");

    // binding buffer to group zero with specific bindings
    let bind_group = create_bind_group(
        device,
        &compute_pipeline,
        [
            (0, storage_buffer_x.as_entire_binding()),
            (1, storage_buffer_y.as_entire_binding()),
            (2, storage_buffer_sizes.as_entire_binding()),
            (3, storage_buffer_out.as_entire_binding()),
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
        cpass.dispatch_workgroups(x.len() as u32, y.len() as u32, 1);
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
    let x = [1, 2, 3, 4];
    let y = [1, 2, 3, 4];
    let n = 2;

    let result = execute_shader(&x, &y, n, &device, &queue).await?;
    println!("x: {:?}, y: {:?}", x, y);
    println!("{:?}", result);
    Ok(())
}
