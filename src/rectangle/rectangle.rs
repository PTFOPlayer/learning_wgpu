use std::mem;

use wgpu::{
    util::DeviceExt, BufferAddress, Device, LoadOp, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration, VertexAttribute, VertexBufferLayout, VertexStepMode
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

use crate::Error;

pub async fn execute_rectangle() -> Result<(), Error> {
    let event_loop = EventLoop::new()?;
    let window = winit::window::WindowBuilder::new().build(&event_loop)?;

    let mut size = window.inner_size();
    size.width = 1;
    size.height = 1;

    let instance = wgpu::Instance::default();

    let surface = instance.create_surface(&window)?;

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .ok_or(Error::AdapterAquasitionError)?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await?;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .unwrap();

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vertex_main",
            buffers: &[Vertex::decriptor()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fragment_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    surface.configure(&device, &config);


    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&VERTS),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let window = &window;

    event_loop
        .run(move |event, target| {
            let _ = (&instance, &adapter, &shader, &pipeline_layout);

            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(s) => {
                        handle_resize(s, &device, window, &surface, &mut config)
                    }
                    WindowEvent::RedrawRequested => {
                        let frame = surface
                            .get_current_texture()
                            .expect("Failed to acquire next swap chain texture");
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });
                        {
                            let mut rpass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: None,
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: LoadOp::Clear(wgpu::Color::default()),
                                            store: StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });
                            rpass.set_pipeline(&render_pipeline);
                            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                            rpass.draw(0..6, 0..1);
                        }

                        queue.submit(Some(encoder.finish()));
                        frame.present();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();

    Ok(())
}

fn handle_resize(
    new_size: PhysicalSize<u32>,
    device: &Device,
    window: &Window,
    surface: &Surface,
    config: &mut SurfaceConfiguration,
) {
    config.width = new_size.width.max(1);
    config.height = new_size.height.max(1);
    surface.configure(&device, &config);
    window.request_redraw();
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

macro_rules! v4xyz {
    ($x:expr, $y:expr, $z:expr) => {
        Vec4 {
            x: $x,
            y: $y,
            z: $z,
            w: 1.0,
        }
    };
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: Vec4,
    color: Vec4,
}

impl Vertex {
    const fn new(position: Vec4, color: Vec4) -> Self {
        Self { position, color }
    }

    const ATTRIBUTES: [VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];

    const fn decriptor() -> VertexBufferLayout<'static> { 
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

const VERTS: [Vertex; 6] = [
    Vertex::new(v4xyz!(-1.0, -1.0, 0.0), v4xyz!(1.0, 0.0, 0.0)), // a
    Vertex::new(v4xyz!(1.0, -1.0, 0.0), v4xyz!(0.0, 1.0, 0.0)),  // b
    Vertex::new(v4xyz!(-1.0, 1.0, 0.0), v4xyz!(0.0, 0.0, 1.0)),  // d
    Vertex::new(v4xyz!(-1.0, 1.0, 0.0), v4xyz!(0.0, 0.0, 1.0)),  // d
    Vertex::new(v4xyz!(1.0, -1.0, 0.0), v4xyz!(0.0, 1.0, 0.0)),  // b
    Vertex::new(v4xyz!(1.0, 1.0, 0.0), v4xyz!(1.0, 0.0, 0.0)),   // c
];
