use crate::graphics;
use crate::input;
use crate::camera::{Camera, CameraUniform};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::DeviceEvent;
use winit::event::{WindowEvent};
use winit::window::Window;

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

    pub input_state: input::InputState,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_uniform_buffer: wgpu::Buffer,

    selected_obj: u32,
    toggle_cooldown: f64,
    pub delta_time: f64,

    depth_texture: (wgpu::TextureView, wgpu::Sampler, wgpu::Texture)
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

        let render_pipeline = graphics::build_pipeline(&[&bind_group_layout], &device, &shader, &config);

        let camera = Camera::new(
            (0.5, 0.5, 1.0).into(),
            270.0,
            0.0,
            config.width as f32 / config.height as f32,
            90.0,
            0.1,
            100.0,
            0.05,
            5.0
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let obj1 = RenderObject {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices_obj1"),
                contents: bytemuck::cast_slice(&[
                    graphics::Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0] }, // 0
                    graphics::Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0] }, // 1
                    graphics::Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0] }, // 2
                    graphics::Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0] }, // 3
                    graphics::Vertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0] }, // 4
                    graphics::Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 0.0] }, // 5
                    graphics::Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] }, // 6
                    graphics::Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] }, // 7
                ]),
                usage: wgpu::BufferUsages::VERTEX
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices_obj1"),
                contents: bytemuck::cast_slice(&[
                    0u16, 1, 2,
                    1, 3, 2,
                    1, 5, 3,
                    5, 7, 3,
                    0, 4, 1,
                    4, 5, 1,
                    5, 4, 7,
                    4, 6, 7,
                    4, 0, 6,
                    0, 2, 6,
                    2, 3, 6,
                    3, 7, 6
                ]),
                usage: wgpu::BufferUsages::INDEX
            }),
            num_indices: 36,
            bind_group: graphics::build_bind_group(&bind_group_layout, include_bytes!("../res/tex/bu.png"), "texture_obj1", &device, &queue, vec![&camera_uniform_buffer])
        };

        let obj2 = RenderObject {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices_obj2"),
                contents: bytemuck::cast_slice(&[
                    graphics::Vertex { position: [0.0, 0.5, 0.0], tex_coords: [0.5, 0.0] },
                    graphics::Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] },
                    graphics::Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [1.0, 1.0] },
                    graphics::Vertex { position: [0.5, -0.5, 0.5], tex_coords: [0.0, 1.0] },
                    graphics::Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] },
                ]),
                usage: wgpu::BufferUsages::VERTEX
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices_obj2"),
                contents: bytemuck::cast_slice(&[
                    0u16, 2, 3,
                    0, 1, 2,
                    0, 4, 1,
                    0, 3, 4,
                    3, 2, 4,
                    2, 1, 4
                ]),
                usage: wgpu::BufferUsages::INDEX
            }),
            num_indices: 18,
            bind_group: graphics::build_bind_group(&bind_group_layout, include_bytes!("../res/tex/bu2.png"), "texture_obj2", &device, &queue, vec![&camera_uniform_buffer])
        };
        
        let depth_texture = graphics::create_depth_texture(&device, &config, "global_depth_texture");

        Self {
            surface,
            device,
            queue,
            config,
            size: window.inner_size(),
            clear_color: wgpu::Color { r: 0.0, g: 0.5, b: 0.0, a: 1.0 },
            render_pipeline,
            obj1,
            obj2,
            input_state: input::InputState::new(),
            camera,
            camera_uniform,
            camera_uniform_buffer,
            selected_obj: 0,
            toggle_cooldown: 0.0,
            delta_time: 0.0,
            depth_texture
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, window_event: Option<&WindowEvent>, device_event: Option<&DeviceEvent>, window: &Window) {
        if let Some(event) = window_event {
            match event {
                WindowEvent::KeyboardInput{ input, .. } => {
                    self.input_state.update_keyboard(input);
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
        if let Some(event) = device_event {
            match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.input_state.update_mouse(delta);
                    window.set_cursor_position(PhysicalPosition::new(self.size.width / 2, self.size.height / 2)).expect("Failed to set cursor position");
                }
                _ => {}
            }
        }
    }

    pub fn update(&mut self) {
        if self.input_state.tab_pressed && self.toggle_cooldown <= 0.0 {
            self.selected_obj = match self.selected_obj {
                0 => 1,
                1 => 0,
                _ => 0
            };
            self.toggle_cooldown = 1.0;
        }
        self.toggle_cooldown -= self.delta_time * 5.0;

        let mouse_move = self.input_state.get_unhandled_mouse_move();

        let (offset_x, offset_y) = mouse_move;
        let c = &mut self.clear_color;
        c.r += offset_x / 2500.0;
        c.b += offset_y / 2500.0;
        if c.r > 1.0 { c.r = 1.0; }
        if c.g > 1.0 { c.g = 1.0; }
        if c.b > 1.0 { c.b = 1.0; }
        if c.r < 0.0 { c.r = 0.0; }
        if c.g < 0.0 { c.g = 0.0; }
        if c.b < 0.0 { c.b = 0.0; }

        self.camera.update_pos(self.input_state.get_movement(), self.delta_time as f32);
        self.camera.update_look((mouse_move.0 as f32, mouse_move.1 as f32));
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_uniform_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.0,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true
                    }),
                    stencil_ops: None
                })
            });

            render_pass.set_pipeline(&self.render_pipeline);
            match self.selected_obj {
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