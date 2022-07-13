use crate::graphics;
use crate::camera::{Camera, CameraUniform};
use wgpu::util::DeviceExt;
use winit::event::{WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};

pub struct App {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    obj1: RenderObject,
    obj2: RenderObject,
    pub input_state: InputState,
    camera: Camera,
    camera_uniform: CameraUniform,
}

pub struct InputState {
    pub space_pressed: bool,
    pub toggle_cooldown: f32,
    pub obj: u32
}

struct RenderObject {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32,
    bind_group: wgpu::BindGroup
}

impl App {
    pub fn new(window: &winit::window::Window) -> Self {
        let (surface, device, queue, config, shader) = graphics::create_wgpu_context(window);
        
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ],
            label: Some("global_bind_group_layout")
        });

        let render_pipeline = App::build_pipeline(&[&bind_group_layout], &device, &shader, &config);

        let camera = Camera::new(
            (0.5, 0.5, 1.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            90.0,
            0.1,
            100.0
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let obj1 = RenderObject {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices_obj1"),
                contents: bytemuck::cast_slice(&[
                    graphics::Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] },
                    graphics::Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] },
                    graphics::Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] },
                    graphics::Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] },
                ]),
                usage: wgpu::BufferUsages::VERTEX
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices_obj1"),
                contents: bytemuck::cast_slice(&[
                    0u16, 1, 2,
                    1, 3, 2
                ]),
                usage: wgpu::BufferUsages::INDEX
            }),
            num_indices: 6,
            bind_group: graphics::build_bind_group(&bind_group_layout, include_bytes!("../res/tex/bu.png"), "texture_obj1", &device, &queue, vec![&camera_buffer])
        };

        let obj2 = RenderObject {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices_obj2"),
                contents: bytemuck::cast_slice(&[
                    graphics::Vertex { position: [0.0, 0.5, 0.0], tex_coords: [0.5, 0.0] },
                    graphics::Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] },
                    graphics::Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] },
                ]),
                usage: wgpu::BufferUsages::VERTEX
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices_obj2"),
                contents: bytemuck::cast_slice(&[
                    0u16, 1, 2,
                ]),
                usage: wgpu::BufferUsages::INDEX
            }),
            num_indices: 3,
            bind_group: graphics::build_bind_group(&bind_group_layout, include_bytes!("../res/tex/bu2.png"), "texture_obj2", &device, &queue, vec![&camera_buffer])
        };

        Self {
            surface,
            device,
            queue,
            config,
            size: window.inner_size(),
            clear_color: wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            render_pipeline,
            obj1,
            obj2,
            input_state: InputState { space_pressed: false, toggle_cooldown: 1.0, obj: 0 },
            camera,
            camera_uniform
        }
    }

    fn build_pipeline(bind_group_layouts: &[&wgpu::BindGroupLayout], device: &wgpu::Device, shader: &wgpu::ShaderModule, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("main_pipeline_layout"),
            bind_group_layouts,
            push_constant_ranges: &[]
        });
    
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("main_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    graphics::Vertex::desc()
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

        render_pipeline
    }

    

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, ..} => {
                self.clear_color.r = position.x / self.size.width as f64;
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

    pub fn update(&mut self) {

    }

    pub fn render(&mut self, obj: u32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
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
            match obj {
                0 => App::render_obj(&mut render_pass, &self.obj1),
                1 => App::render_obj(&mut render_pass, &self.obj2),
                _ => {}   
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn render_obj<'a>(render_pass: &mut wgpu::RenderPass<'a>, obj: &'a RenderObject) {
        render_pass.set_bind_group(0, &obj.bind_group, &[]);
        render_pass.set_vertex_buffer(0, obj.vertices.slice(..));
        render_pass.set_index_buffer(obj.indices.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..obj.num_indices, 0, 0..1);
    }
}