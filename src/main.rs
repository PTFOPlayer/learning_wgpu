use std::io;

use crate::error::Error;
use dot_product::dot_product::execute_dot_product;
use matrix_dot_product::matrix_dot_product::execute_matrix_dot_product;
use rectangle::rectangle::execute_rectangle;
use saxpy::saxpy::execute_saxpy;
use transpose::transpose::execute_transpose;
use triangle::triangle::execute_triangle;
use wgpu::{Device, Queue, RequestAdapterOptions};

pub mod dot_product;
pub mod error;
pub mod helpers;
pub mod matrix_dot_product;
pub mod rectangle;
pub mod saxpy;
pub mod transpose;
pub mod triangle;
fn main() -> Result<(), Error> {
    // let (device, queue) = smol::block_on(init_device())?;

    print!(
        r#"
Select shader:
    (1) saxpy
    (2) vec dot product
    (3) transpose
    (4) matrix dot product
"#
    );

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let maybe_u32 = buffer[..buffer.len() - 1].parse::<u32>();

    match maybe_u32 {
        Ok(a @ 1..=4) => {
            let (device, queue) = smol::block_on(init_compute_device())?;
            match a {
                1 => smol::block_on(execute_saxpy(device, queue))?,
                2 => smol::block_on(execute_dot_product(device, queue))?,
                3 => smol::block_on(execute_transpose(device, queue))?,
                4 => smol::block_on(execute_matrix_dot_product(device, queue))?,
                _ => println!("unreachable!"),
            };
        }
        Ok(5) => smol::block_on(execute_triangle())?,
        Ok(6) => smol::block_on(execute_rectangle())?,

        _ => panic!("incorrect input"),
    };

    Ok(())
}

/// init device
/// Generates WGPU instance and aquires GPU
pub async fn init_compute_device() -> Result<(Device, Queue), Error> {
    let instance = wgpu::Instance::default();

    let adapter = match instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
    {
        Some(adapter) => adapter,
        None => return Err(Error::AdapterAquasitionError),
    };

    Ok(adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await?)
}
