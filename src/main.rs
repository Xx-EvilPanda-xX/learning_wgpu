use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}
};

fn main() {
    run_app();
}

fn run_app() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_inner_size(winit::dpi::PhysicalSize::new(1280, 720)).build(&event_loop).unwrap();
    
    let mut app = App::new(&window);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { 
            ref event,
            window_id      
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput { 
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                },
                .. 
            } => *control_flow = ControlFlow::Exit,
            _ => app.input(event)
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            app.update();
            match app.render(app.input_state.obj) {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost) => app.resize(app.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e)
            }
        }
        Event::MainEventsCleared => {
            if app.input_state.space_pressed && app.input_state.toggle_cooldown <= 0.0 {
                app.input_state.obj = match app.input_state.obj {
                    0 => 1,
                    1 => 0,
                    _ => 0
                };
                app.input_state.toggle_cooldown = 1.0;
            }
            window.request_redraw();
            app.input_state.toggle_cooldown -= 0.01;
        }
        _ => {}
    });
}
struct App {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    obj1: RenderObject,
    obj2: RenderObject,
    input_state: InputState
}

struct InputState {
    space_pressed: bool,
    toggle_cooldown: f32,
    obj: u32
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3]
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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

struct RenderObject {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32
}

impl App {
    fn new(window: &Window) -> Self {
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


        let obj1 = RenderObject {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Buffer bu"),
                contents: bytemuck::cast_slice(&[
                    Vertex { position: [0.5, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
                    Vertex { position: [-0.5, 0.5, 0.0], color: [0.0, 1.0, 0.0] },
                    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
                    Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
                ]),
                usage: wgpu::BufferUsages::VERTEX
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indioces"),
                contents: bytemuck::cast_slice(&[
                    0u16, 1, 2,
                    1, 3, 2
                ]),
                usage: wgpu::BufferUsages::INDEX
            }),
            num_indices: 6
        };

        let obj2 = RenderObject {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Buffer bu"),
                contents: bytemuck::cast_slice(&[
                    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
                    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
                    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
                ]),
                usage: wgpu::BufferUsages::VERTEX
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indioces"),
                contents: bytemuck::cast_slice(&[
                    0u16, 1, 2,
                ]),
                usage: wgpu::BufferUsages::INDEX
            }),
            num_indices: 3
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color: wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            render_pipeline,
            obj1,
            obj2,
            input_state: InputState { space_pressed: false, toggle_cooldown: 1.0, obj: 0 }
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, ..} => {
                self.clear_color.g = position.x / self.size.width as f64;
                self.clear_color.b = position.y / self.size.height as f64;
            }
            WindowEvent::KeyboardInput { 
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                },
                ..
            } => {
                match state {
                    ElementState::Pressed => self.input_state.space_pressed = true,
                    ElementState::Released => self.input_state.space_pressed = false
                }
            }
            WindowEvent::Resized(new_size) => {
                self.resize(*new_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(**new_inner_size);
            }
            _ => {}
        }
    }

    fn update(&mut self) {

    }

    fn render(&mut self, obj: u32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder thingy")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pas thiny"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true
                    }
                })],
                depth_stencil_attachment: None
            });

            render_pass.set_pipeline(&self.render_pipeline);
            if obj == 0 {
                App::render_obj(&mut render_pass, &self.obj1);
            } else if obj == 1 {
               App::render_obj(&mut render_pass, &self.obj2);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn render_obj<'a>(render_pass: &mut wgpu::RenderPass<'a>, obj: &'a RenderObject) {
        render_pass.set_vertex_buffer(0, obj.vertices.slice(..));
        render_pass.set_index_buffer(obj.indices.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..obj.num_indices, 0, 0..1);
    }
}