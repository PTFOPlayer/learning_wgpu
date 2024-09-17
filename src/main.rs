use dot_product::dot_product::execute_dot_product;
use saxpy::saxpy::execute_saxpy;
use transpose::transpose::execute_transpose;
use wgpu::RequestDeviceError;

pub mod saxpy;
pub mod dot_product;
pub mod helpers;
pub mod transpose;

#[derive(Debug)]
pub enum Error {
    AdapterAquasitionError,
    DeviceCreationError(RequestDeviceError),
    ExecutionError,
}

fn main() -> Result<(), Error> {
    smol::block_on(execute_saxpy())?;
    smol::block_on(execute_dot_product())?;
    smol::block_on(execute_transpose())?;

    Ok(())
}
