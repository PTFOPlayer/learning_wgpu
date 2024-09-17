use std::borrow::Cow;

use wgpu::{
    util::DeviceExt, BindGroup, BindGroupEntry, BindingResource, Buffer, BufferUsages,
    ComputePipeline, Device, Queue, RequestAdapterOptions,
};

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

pub fn create_pipeline(device: &Device, module: &str, entry_point: &str) -> ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        // loading module, path for module is relative
        module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(module)),
        }),
        entry_point,
        compilation_options: Default::default(),
        cache: None,
    })
}

pub fn create_bind_group(
    device: &Device,
    compute_pipeline: &ComputePipeline,
    entries: Vec<(u32, BindingResource)>,
) -> BindGroup {
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &entries
            .into_iter()
            .map(|(binding, resource)| BindGroupEntry { binding, resource })
            .collect::<Vec<_>>(),
    })
}

pub fn create_storage_buffer<T: bytemuck::Pod>(
    device: &Device,
    slice: &[T],
    usage: BufferUsages,
) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(slice),
        usage,
    })
}

pub fn create_staging_buffer(device: &Device, size: u64) -> Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
