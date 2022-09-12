const WIREFRAME: bool = false;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[derive(Clone)]
pub struct Instance {
    pub trans: cgmath::Vector3<f32>,
    pub rot: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model_mat: RawMatrix,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawMatrix {
    pub mat: [[f32; 4]; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { // position
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // tex coords
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

impl Instance {
    pub fn as_raw(&self) -> InstanceRaw {
        InstanceRaw { 
            model_mat: RawMatrix { 
                mat: (cgmath::Matrix4::from_translation(self.trans) * cgmath::Matrix4::from(self.rot)).into()
            }
        }
    }
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute { // model mat col 1
                    offset: 0 as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute { // model mat col 2
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute { // model mat col 3
                    offset: (size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute { // model mat col 4
                    offset: (size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                }
            ],
        }
    }
}

impl RawMatrix {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        RawMatrix {
            mat: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &super::camera::Camera) {
        self.mat = camera.build_view_proj().into();
    }
}

pub fn create_wgpu_context(
    window: &winit::window::Window,
) -> (
    wgpu::Surface,
    wgpu::Device,
    wgpu::Queue,
    wgpu::SurfaceConfiguration,
    wgpu::ShaderModule,
) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::POLYGON_MODE_LINE,
            limits: wgpu::Limits::default(),
            label: Some("main_device"),
        },
        None,
    ))
    .unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader at shader.wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    (surface, device, queue, config, shader)
}

pub fn build_pipeline(
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("main_pipeline_layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("main_pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc(), InstanceRaw::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: if WIREFRAME {
                wgpu::PolygonMode::Line
            } else {
                wgpu::PolygonMode::Fill
            },
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    render_pipeline
}

pub fn build_bind_group(
    bind_group_layout: &wgpu::BindGroupLayout,
    tex_bytes: &[u8],
    name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    uniforms: Vec<&wgpu::Buffer>,
) -> wgpu::BindGroup {
    let (view, sampler, _) = load_texture(device, queue, tex_bytes, name);

    let mut entries = Vec::new();

    for (i, buffer) in uniforms.iter().enumerate() {
        entries.push(wgpu::BindGroupEntry {
            binding: i as u32,
            resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
        });
    }

    entries.push(wgpu::BindGroupEntry {
        binding: uniforms.len() as u32,
        resource: wgpu::BindingResource::TextureView(&view),
    });

    entries.push(wgpu::BindGroupEntry {
        binding: uniforms.len() as u32 + 1,
        resource: wgpu::BindingResource::Sampler(&sampler),
    });

    // println!("{:#?}", entries);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &entries,
        label: Some(name),
    });

    bind_group
}

fn load_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[u8],
    name: &str,
) -> (wgpu::TextureView, wgpu::Sampler, wgpu::Texture) {
    let tex_img = image::load_from_memory(data).unwrap();
    let tex_rgba = tex_img.to_rgba8();

    use image::GenericImageView;
    let dims = tex_img.dimensions();

    let tex_size = wgpu::Extent3d {
        width: dims.0,
        height: dims.1,
        depth_or_array_layers: 1,
    };

    let tex = device.create_texture(&wgpu::TextureDescriptor {
        size: tex_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some(name),
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &tex_rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * dims.0),
            rows_per_image: std::num::NonZeroU32::new(dims.1),
        },
        tex_size,
    );

    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    (view, sampler, tex)
}

pub fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    label: &str,
) -> (wgpu::TextureView, wgpu::Sampler, wgpu::Texture) {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    });

    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    (view, sampler, tex)
}
