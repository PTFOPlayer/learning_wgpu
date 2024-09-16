use std::borrow::Cow;

use wgpu::{util::DeviceExt, Device, Queue, RequestAdapterOptions, RequestDeviceError};

#[derive(Debug)]
enum Error {
    AdapterAquasitionError,
    DeviceCreationError(RequestDeviceError),
    ExecutionError,
}

/// init device
/// Generates WGPU instance and aquires GPU
async fn init_device() -> Result<(Device, Queue), Error> {
    let instance = wgpu::Instance::default();

    let adapter = match instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
    {
        Some(adapter) => adapter,
        None => return Err(Error::AdapterAquasitionError),
    };

    match adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
    {
        Ok(res) => Ok(res),
        Err(error) => Err(Error::DeviceCreationError(error)),
    }
}

// executes shader with given parameters
async fn execute_shader(
    a: i32,
    x: &[i32],
    y: &[i32],
    device: &Device,
    queue: &Queue,
) -> Result<Vec<i32>, Error> {
    // loading module
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let size_x = size_of_val(x) as wgpu::BufferAddress;
    // return buffer
    // MAP_READ allows for reading it
    // COPY_DST allows for it to be desetination of cpy
    let staging_buffer_x = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: size_x,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // buffer that is avaliable for GPU
    let storage_buffer_x = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(x),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    // buffer that is avaliable for GPU
    let storage_buffer_y = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(y),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    // buffer that is avaliable for GPU
    let storage_buffer_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(&[a]),
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
                resource: storage_buffer_a.as_entire_binding(),
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

async fn async_execute() -> Result<(), Error> {
    let x = [1, 2, 3, 4];
    let y = [4, 3, 2, 1];
    let a = 10;

    let (device, queue) = init_device().await?;

    let result = execute_shader(a, &x, &y, &device, &queue).await?;

    println!("{:?}", result);
    Ok(())
}

fn main() -> Result<(), Error> {
    smol::block_on(async_execute())?;

    Ok(())
}
