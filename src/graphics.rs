#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3]
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3
                }
            ]
        }
    }
}

pub fn create_wgpu_context(window: &winit::window::Window) -> (wgpu::Surface, wgpu::Device, wgpu::Queue, wgpu::SurfaceConfiguration, wgpu::RenderPipeline) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe {
        instance.create_surface(window)
    };
    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false 
        }
    )).unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: Some("Device thingy")
        },
        None
    )).unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader bu"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline layout bububub"),
        bind_group_layouts: &[],
        push_constant_ranges: &[]
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("the actual pipline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                Vertex::desc()
            ]
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL
            })]
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false
        },
        multiview: None
    });

    (surface, device, queue, config, render_pipeline)
}