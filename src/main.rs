use std::io;

use dot_product::dot_product::execute_dot_product;
use saxpy::saxpy::execute_saxpy;
use transpose::transpose::execute_transpose;
use wgpu::{Device, Queue, RequestAdapterOptions, RequestDeviceError};

pub mod dot_product;
pub mod helpers;
pub mod saxpy;
pub mod transpose;

#[derive(Debug)]
pub enum Error {
    AdapterAquasitionError,
    DeviceCreationError(RequestDeviceError),
    ExecutionError,
    IoError,
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        Error::IoError
    }
}

fn main() -> Result<(), Error> {
    let (device, queue) = smol::block_on(init_device())?;

    print!(
        r#"
Select shader:
    (1) saxpy
    (2) saxpy
    (3) saxpy
"#
    );

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    match buffer[..buffer.len() - 1].parse::<u32>() {
        Ok(1) => smol::block_on(execute_saxpy(device, queue))?,
        Ok(2) => smol::block_on(execute_dot_product(device, queue))?,
        Ok(3) => smol::block_on(execute_transpose(device, queue))?,
        Ok(_) | Err(_) => {
            println!("wrong input")
        }
    }

    Ok(())
}

/// init device
/// Generates WGPU instance and aquires GPU
pub async fn init_device() -> Result<(Device, Queue), Error> {
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
