use wgpu::{Device, Queue, RequestAdapterOptions};

use crate::Error;


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
