use wgpu::{BufferUsages, Device, Queue};

use crate::{
    helpers::{
        create_bind_group, create_pipeline, create_staging_buffer, create_storage_buffer,
    },
    Error,
};

// executes shader with given parameters
async fn execute_shader(x: &[i32], device: &Device, queue: &Queue) -> Result<Vec<i32>, Error> {
    let out = vec![0; x.len()];
    let out_slice = out.as_slice();
    let size = size_of_val(out_slice) as wgpu::BufferAddress;
    // return buffer
    // MAP_READ allows for reading it
    // COPY_DST allows for it to be desetination of cpy
    let staging_buffer_out = create_staging_buffer(device, size);

    // output buffer that is avaliable for GPU
    let storage_buffer_out = create_storage_buffer(
        device,
        x,
        BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
    );

    // buffer that is avaliable for GPU
    let storage_buffer_x = create_storage_buffer(device, x, BufferUsages::STORAGE);

    // creation of compute pipeline with entrypoint "main"
    let compute_pipeline = create_pipeline(device, include_str!("shader.wgsl"), "main");

    // binding buffer to group zero with specific bindings
    let bind_group = create_bind_group(
        device,
        &compute_pipeline,
        [
            (0, storage_buffer_x.as_entire_binding()),
            (1, storage_buffer_out.as_entire_binding()),
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
        cpass.insert_debug_marker("transposition");
        cpass.dispatch_workgroups(x.len() as u32 / 4, x.len() as u32 / 4, 1);
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

pub async fn execute_transpose(device: Device, queue: Queue) -> Result<(), Error> {
    #[rustfmt::skip]
    let x = [
        1,  2,  3,  4, 
        5,  6,  7,  8,
        9,  10, 11, 12,
        13, 14, 15, 16
    ];

    let result = execute_shader(&x, &device, &queue).await?;

    println!("{:?}", result);
    Ok(())
}
