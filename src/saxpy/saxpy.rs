use wgpu::{BufferUsages, Device, Queue};

use crate::{
    helpers::{
        create_bind_group, create_pipeline, create_staging_buffer, create_storage_buffer,
        init_device,
    },
    Error,
};

// executes shader with given parameters
async fn execute_shader(
    a: i32,
    x: &[i32],
    y: &[i32],
    device: &Device,
    queue: &Queue,
) -> Result<Vec<i32>, Error> {
    let size_x = size_of_val(x) as wgpu::BufferAddress;
    // return buffer
    // MAP_READ allows for reading it
    // COPY_DST allows for it to be desetination of cpy
    let staging_buffer_x = create_staging_buffer(device, size_x);

    // buffer that is avaliable for GPU
    let storage_buffer_x = create_storage_buffer(
        device,
        x,
        BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
    );

    // buffer that is avaliable for GPU
    let storage_buffer_y = create_storage_buffer(device, y, BufferUsages::STORAGE);

    // buffer that is avaliable for GPU
    let storage_buffer_a = create_storage_buffer(device, &[a], BufferUsages::STORAGE);

    // creation of compute pipeline with entrypoint "main"
    let compute_pipeline = create_pipeline(device, include_str!("shader.wgsl"), "main");

    // binding buffer to group zero with specific bindings
    let bind_group = create_bind_group(
        device,
        &compute_pipeline,
        vec![
            (0, storage_buffer_x.as_entire_binding()),
            (1, storage_buffer_y.as_entire_binding()),
            (2, storage_buffer_a.as_entire_binding()),
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
        cpass.insert_debug_marker("saxpy");
        cpass.dispatch_workgroups(x.len() as u32, 1, 1);
    }

    // copy result
    encoder.copy_buffer_to_buffer(&storage_buffer_x, 0, &staging_buffer_x, 0, size_x);

    queue.submit(Some(encoder.finish()));

    // sending data back to host
    let buffer_slice = staging_buffer_x.slice(..);

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
        staging_buffer_x.unmap();

        return Ok(result);
    }

    Err(Error::ExecutionError)
}

pub async fn execute_saxpy() -> Result<(), Error> {
    let x = [1, 2, 3, 4];
    let y = [4, 3, 2, 1];
    let a = 10;

    let (device, queue) = init_device().await?;

    let result = execute_shader(a, &x, &y, &device, &queue).await?;

    println!("{:?}", result);
    Ok(())
}
