use dot_product::dot_product::execute_dot_product;
use saxpy::saxpy::execute_saxpy;
use wgpu::RequestDeviceError;

pub mod saxpy;
pub mod dot_product;
pub mod helpers;

#[derive(Debug)]
pub enum Error {
    AdapterAquasitionError,
    DeviceCreationError(RequestDeviceError),
    ExecutionError,
}

fn main() -> Result<(), Error> {
    smol::block_on(execute_saxpy())?;
    smol::block_on(execute_dot_product())?;

    Ok(())
}
