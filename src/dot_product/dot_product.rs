use std::borrow::Cow;

use wgpu::{util::DeviceExt, Device, Queue};

use crate::{helpers::init_device, Error};

// executes shader with given parameters
async fn execute_shader(
    out: &[i32],
    x: &[i32],
    y: &[i32],
    device: &Device,
    queue: &Queue,
) -> Result<Vec<i32>, Error> {
    // loading module, path for module is relative
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let size = size_of_val(out) as wgpu::BufferAddress;
    // return buffer
    // MAP_READ allows for reading it
    // COPY_DST allows for it to be desetination of cpy
    let staging_buffer_out = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // output buffer that is avaliable for GPU
    let storage_buffer_out = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(out),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    // buffer that is avaliable for GPU
    let storage_buffer_x = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(x),
        usage: wgpu::BufferUsages::STORAGE,
    });

    // buffer that is avaliable for GPU
    let storage_buffer_y = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(y),
        usage: wgpu::BufferUsages::STORAGE,
    });

    // creation of compute pipeline with entrypoint "main"
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &module,
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None,
    });

    // binding buffer to group zero with specific bindings
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer_x.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: storage_buffer_y.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: storage_buffer_out.as_entire_binding(),
            },
        ],
    });

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
        cpass.insert_debug_marker("compute collatz iterations");
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

pub async fn execute_dot_product() -> Result<(), Error> {
    let x = [1, 2, 3, 4];
    let y = [1, 2, 3, 4];
    let out = [0; 16];

    let (device, queue) = init_device().await?;

    let result = execute_shader(&out, &x, &y, &device, &queue).await?;

    println!("{:?}", result);
    Ok(())
}
