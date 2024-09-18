use std::borrow::Cow;

use wgpu::{
    util::DeviceExt, BindGroup, BindGroupEntry, BindingResource, Buffer, BufferUsages,
    ComputePipeline, Device,
};

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

pub fn create_bind_group<const SIZE: usize>(
    device: &Device,
    compute_pipeline: &ComputePipeline,
    entries: [(u32, BindingResource); SIZE],
) -> BindGroup {
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &entries.map(|(binding, resource)| BindGroupEntry { binding, resource }),
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
