use crate::graphics;
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
    pub input_state: InputState
}

pub struct InputState {
    pub space_pressed: bool,
    pub toggle_cooldown: f32,
    pub obj: u32
}

struct RenderObject {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32
}

impl App {
    pub fn new(window: &winit::window::Window) -> Self {
        let (surface, device, queue, config, render_pipeline) = graphics::create_wgpu_context(window);

        let obj1 = RenderObject {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Buffer bu"),
                contents: bytemuck::cast_slice(&[
                    graphics::Vertex { position: [0.5, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
                    graphics::Vertex { position: [-0.5, 0.5, 0.0], color: [0.0, 1.0, 0.0] },
                    graphics::Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
                    graphics::Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
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
                    graphics::Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
                    graphics::Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
                    graphics::Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
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
            size: window.inner_size(),
            clear_color: wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            render_pipeline,
            obj1,
            obj2,
            input_state: InputState { space_pressed: false, toggle_cooldown: 1.0, obj: 0 }
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

    pub fn input(&mut self, event: &WindowEvent) {
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

    pub fn update(&mut self) {

    }

    pub fn render(&mut self, obj: u32) -> Result<(), wgpu::SurfaceError> {
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