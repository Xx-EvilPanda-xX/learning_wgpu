use crate::camera::Camera;
use crate::graphics;
use crate::graphics::Instance;
use crate::graphics::RawMatrix;
use crate::input;
use cgmath::InnerSpace;
use cgmath::{Matrix4, Rotation3, SquareMatrix, Vector3};
use log::debug;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::DeviceEvent;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct App {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,

    obj1: (RenderObject, wgpu::BindGroup),
    obj2: (RenderObject, wgpu::BindGroup),
    pythagoras_sphere: (RenderObject, wgpu::BindGroup),
    floor: (RenderObject, wgpu::BindGroup),

    pub input_state: input::InputState,

    camera: Camera,
    camera_uniform: RawMatrix,
    camera_uniform_buffer: wgpu::Buffer,

    selected_obj: u32,
    cooldowns: (f64, f64),
    pub delta_time: f64,

    depth_texture: (wgpu::TextureView, wgpu::Sampler, wgpu::Texture),
    intial_instant: std::time::Instant,
}

struct RenderObject {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    model_buf: wgpu::Buffer,
    is_instanced_buf: wgpu::Buffer,
    num_indices: u32,
    instances_buffer: Option<wgpu::Buffer>,
    num_instances: Option<u32>,
    shown_instances: Option<u32>,
}

pub const INSTANCED_ROWS: usize = 50;
pub const INSTANCED_COLS: usize = 50;
pub const INSTANCE_SPACING: f32 = 3.0;
const SPHERE_INSTANCED_ROWS: usize = 10;
const SPHERE_INSTANCED_COLS: usize = 10;
const SPHERE_INSTANCE_SPACING: f32 = 15.0;
const FLOOR_Y: f32 = -25.0;

impl App {
    pub fn new(window: &winit::window::Window) -> Self {
        let (surface, device, queue, config, shader) = graphics::create_wgpu_context(window);
        let bind_group_layout = build_bind_group_layout(&device);
        let render_pipeline = graphics::build_pipeline(&[&bind_group_layout], &device, &shader, &config);
        let camera = Camera::new(
            (0.0, 0.0, 0.0).into(),
            45.0,
            0.0,
            config.width as f32 / config.height as f32,
            5.0
        );

        let mut camera_uniform = RawMatrix::new();
        camera_uniform.update_view_proj(&camera);

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let rot_instances = (0..INSTANCED_ROWS)
            .flat_map(|x| {
                (0..INSTANCED_COLS).map(move |z| Instance {
                    trans: Vector3::new(
                        x as f32 * INSTANCE_SPACING,
                        0.0,
                        z as f32 * INSTANCE_SPACING,
                    ),
                    rot: cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg((x * 10) as f32 + (z * 10) as f32),
                    ),
                })
            })
            .collect::<Vec<_>>();

        let sphere_instances = (0..SPHERE_INSTANCED_ROWS)
            .flat_map(|x| {
                (0..SPHERE_INSTANCED_COLS).map(move |z| Instance {
                    trans: Vector3::new(
                        x as f32 * SPHERE_INSTANCE_SPACING,
                        0.0,
                        z as f32 * SPHERE_INSTANCE_SPACING,
                    ),
                    rot: cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    ),
                })
            })
            .collect::<Vec<_>>();

        let obj1 = build_obj1(&device, &rot_instances);
        let obj2 = build_obj2(&device, &rot_instances);
        let floor = build_floor(&device);
        let pythagoras_sphere = build_sphere(&device, &sphere_instances);

        let create_bind_group = |model_buf, is_instanced_buf, tex_path, tex_name| graphics::build_bind_group(
            &bind_group_layout,
            &std::fs::read(tex_path).expect("Failed to load texture"),
            tex_name,
            &device,
            &queue,
            vec![&camera_uniform_buffer, model_buf, is_instanced_buf],
        );

        let obj1_bind_group = create_bind_group(&obj1.model_buf, &obj1.is_instanced_buf, "res/tex/tex4.jpg", "texture_obj1");
        let obj2_bind_group = create_bind_group(&obj2.model_buf, &obj2.is_instanced_buf,"res/tex/tex6.png", "texture_obj2");
        let floor_bind_group = create_bind_group(&floor.model_buf, &floor.is_instanced_buf,"res/tex/floor.png", "texture_floor");
        let pythagoras_sphere_bind_group = create_bind_group(&pythagoras_sphere.model_buf, &pythagoras_sphere.is_instanced_buf,"res/tex/bricks.jpg", "texture_sphere");

        let depth_texture = graphics::create_depth_texture(&device, &config, "global_depth_texture");

        Self {
            surface,
            device,
            queue,
            config,
            size: window.inner_size(),
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.25,
                b: 0.0,
                a: 1.0,
            },
            render_pipeline,
            obj1: (obj1, obj1_bind_group),
            obj2: (obj2, obj2_bind_group),
            floor: (floor, floor_bind_group),
            pythagoras_sphere: (pythagoras_sphere, pythagoras_sphere_bind_group),
            input_state: input::InputState::new(),
            camera,
            camera_uniform,
            camera_uniform_buffer,
            selected_obj: 1,
            cooldowns: (0.0, 0.0),
            delta_time: 0.0,
            depth_texture,
            intial_instant: std::time::Instant::now(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                graphics::create_depth_texture(&self.device, &self.config, "global_depth_texture");
            self.camera
                .set_aspect(self.config.width as f32 / self.config.height as f32);
        }
    }

    pub fn input(
        &mut self,
        window_event: Option<&WindowEvent>,
        device_event: Option<&DeviceEvent>,
        window: &Window,
        focused: bool,
    ) {
        if let Some(event) = window_event {
            match event {
                WindowEvent::KeyboardInput { input, .. } if focused => {
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
                DeviceEvent::MouseMotion { delta } if focused => {
                    self.input_state.update_mouse(delta);
                    window
                        .set_cursor_position(PhysicalPosition::new(
                            self.size.width / 2,
                            self.size.height / 2,
                        ))
                        .expect("Failed to set cursor position");
                }
                _ => {}
            }
        }
    }

    pub fn update(&mut self) {
        if self.input_state.tab_pressed && self.cooldowns.0 <= 0.0 {
            self.selected_obj = match self.selected_obj {
                0 => 1,
                1 => 0,
                _ => 0,
            };
            self.cooldowns.0 = 1.0;
        }

        if let (
            Some(shown_instances1),
            Some(shown_instances2),
            Some(num_instances1),
            Some(num_instances2),
        ) = (
            &mut self.obj1.0.shown_instances,
            &mut self.obj2.0.shown_instances,
            &self.obj1.0.num_instances,
            &self.obj2.0.num_instances,
        ) {
            if self.input_state.up_pressed && self.cooldowns.1 <= 0.75 {
                match self.selected_obj {
                    0 if *shown_instances1 < *num_instances1 => *shown_instances1 += 1,
                    1 if *shown_instances2 < *num_instances2 => *shown_instances2 += 1,
                    _ => {}
                }
                self.cooldowns.1 = 1.0;
            }

            if self.input_state.down_pressed && self.cooldowns.1 <= 0.75 {
                match self.selected_obj {
                    0 if *shown_instances1 > 0 => *shown_instances1 -= 1,
                    1 if *shown_instances2 > 0 => *shown_instances2 -= 1,
                    _ => {}
                }
                self.cooldowns.1 = 1.0;
            }
        }

        self.cooldowns.0 -= self.delta_time * 5.0;
        self.cooldowns.1 -= self.delta_time * 5.0;

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

        self.camera.update_pos(self.delta_time as f32, &self.input_state);
        self.camera.update_look(
            (mouse_move.0 as f32, mouse_move.1 as f32),
            self.delta_time as f32,
        );
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let now = std::time::Instant::now()
            .duration_since(self.intial_instant)
            .as_secs_f32();
        let sin = now.sin();
        let cos = now.cos();

        let obj1_model = Matrix4::from_angle_x(cgmath::Rad { 0: now })
            * Matrix4::from_angle_y(cgmath::Rad { 0: now })
            * Matrix4::from_angle_z(cgmath::Rad { 0: now });

        let obj2_model = Matrix4::from_translation(Vector3::new(sin * 10.0, sin, cos * 10.0))
            * Matrix4::from_scale(sin.abs() + 1.22);

        let pythagoras_sphere_model = Matrix4::from_translation(Vector3::new(0.0, FLOOR_Y + 5.0, 0.0))
            * Matrix4::from_axis_angle(Vector3::new(1.0, 1.0, 1.0).normalize(), cgmath::Rad { 0: now });

        let write_buffer = |dest, src: Matrix4<f32>| self.queue.write_buffer(
            dest,
            0,
            bytemuck::cast_slice(&[super::graphics::RawMatrix {
                mat: src.into(),
            }]),
        );

        write_buffer(&self.obj1.0.model_buf, obj1_model);
        write_buffer(&self.obj2.0.model_buf, obj2_model);
        write_buffer(&self.pythagoras_sphere.0.model_buf, pythagoras_sphere_model);

        if self.input_state.f_pressed {
            debug!(
                "Player location: {}, {}, {}",
                self.camera.loc.x, self.camera.loc.y, self.camera.loc.z
            );
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.0,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            match self.selected_obj {
                0 => App::render_obj(&mut render_pass, &self.obj1),
                1 => App::render_obj(&mut render_pass, &self.obj2),
                _ => {}
            }
            App::render_obj(&mut render_pass, &self.pythagoras_sphere);
            App::render_obj(&mut render_pass, &self.floor);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn render_obj<'a>(
        render_pass: &mut wgpu::RenderPass<'a>,
        obj: &'a (RenderObject, wgpu::BindGroup),
    ) {
        render_pass.set_bind_group(0, &obj.1, &[]);
        render_pass.set_vertex_buffer(0, obj.0.vertices.slice(..));
        if let Some(ref buf) = obj.0.instances_buffer {
            render_pass.set_vertex_buffer(1, buf.slice(..));
        }
        render_pass.set_index_buffer(obj.0.indices.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(
            0..obj.0.num_indices,
            0,
            0..obj.0.shown_instances.unwrap_or(1),
        );
    }
}

fn build_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry { // view/projection matrix uniform
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry { // model matrix uniform
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry { // is instanced uniform
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry { // texture data
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry { // texture sampler
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("global_bind_group_layout"),
    })
}

fn build_obj1(device: &wgpu::Device, instances: &Vec<Instance>) -> RenderObject {
    RenderObject {
        vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertices_obj1"),
            contents: bytemuck::cast_slice(&[
                graphics::Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0] }, // 0
                graphics::Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0] }, // 1
                graphics::Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0] }, // 2
                graphics::Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0] }, // 3
                graphics::Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [1.0, 0.0] }, // 4
                graphics::Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 0.0] }, // 5
                graphics::Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [1.0, 1.0] }, // 6
                graphics::Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] }, // 7
                graphics::Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0] }, // 8
                graphics::Vertex { position: [0.5, 0.5, -0.5], tex_coords: [0.0, 0.0] }, // 9
                graphics::Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [1.0, 1.0] }, // 10
                graphics::Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 1.0] }, // 11
                graphics::Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [1.0, 0.0] }, // 12
                graphics::Vertex { position: [0.5, 0.5, -0.5], tex_coords: [0.0, 0.0] }, // 13
                graphics::Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] }, // 14
                graphics::Vertex { position: [0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] }, // 15
                graphics::Vertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0] }, // 16
                graphics::Vertex { position: [0.5, 0.5, 0.5], tex_coords: [0.0, 0.0] }, // 17
                graphics::Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] }, // 18
                graphics::Vertex { position: [0.5, -0.5, 0.5], tex_coords: [0.0, 1.0] }, // 19
                graphics::Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 0.0] }, // 20
                graphics::Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 0.0] }, // 21
                graphics::Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] }, // 22
                graphics::Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] }, // 23
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices_obj1"),
            contents: bytemuck::cast_slice(&[
                0u32, 1, 2,
                1, 3, 2,
                4, 5, 6,
                5, 7, 6,
                8, 9, 10,
                9, 11, 10,
                12, 13, 14,
                13, 15, 14,
                16, 17, 18,
                17, 19, 18,
                20, 21, 22,
                21, 23, 22,
            ]),
            usage: wgpu::BufferUsages::INDEX,
        }),
        model_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("model_obj1"),
            contents: bytemuck::cast_slice(&[super::graphics::RawMatrix {
                mat: Matrix4::identity().into(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        is_instanced_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("is_instanced_obj1"),
            contents: bytemuck::cast_slice(&[1u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        num_indices: 36,
        instances_buffer: Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("obj1_instance_buffer"),
                contents: bytemuck::cast_slice(
                    &instances.iter().map(Instance::as_raw).collect::<Vec<_>>(),
                ),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        ),
        num_instances: Some(instances.len() as u32),
        shown_instances: Some((INSTANCED_ROWS * INSTANCED_COLS) as u32),
    }
}

fn build_obj2(device: &wgpu::Device, instances: &Vec<Instance>) -> RenderObject {
    RenderObject {
        vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertices_obj2"),
            contents: bytemuck::cast_slice(&[
                graphics::Vertex { position: [0.0, 0.5, 0.0], tex_coords: [0.5, 0.0] }, // 0
                graphics::Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] }, // 1
                graphics::Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [1.0, 1.0] }, // 2
                graphics::Vertex { position: [0.5, -0.5, 0.5], tex_coords: [0.0, 1.0] }, // 3
                graphics::Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] }, // 4
                graphics::Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] }, // 5
                graphics::Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 0.0] }, // 6
                graphics::Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 0.0] }, // 7
                graphics::Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] }, // 8
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices_obj2"),
            contents: bytemuck::cast_slice(&[
                0u32, 2, 3,
                0, 1, 2,
                0, 4, 1,
                0, 3, 4,
                7, 6, 8,
                6, 5, 8,
            ]),
            usage: wgpu::BufferUsages::INDEX,
        }),
        model_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("model_obj2"),
            contents: bytemuck::cast_slice(&[super::graphics::RawMatrix {
                mat: Matrix4::identity().into(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        is_instanced_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("is_instanced_obj2"),
            contents: bytemuck::cast_slice(&[1u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        num_indices: 18,
        instances_buffer: Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("obj2_instance_buffer"),
                contents: bytemuck::cast_slice(
                    &instances.iter().map(Instance::as_raw).collect::<Vec<_>>(),
                ),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        ),
        num_instances: Some(instances.len() as u32),
        shown_instances: Some((INSTANCED_ROWS * INSTANCED_COLS) as u32),
    }
}

fn build_floor(device: &wgpu::Device) -> RenderObject {
    RenderObject {
        vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertices_floor"),
            contents: bytemuck::cast_slice(&[
                graphics::Vertex {
                    position: [0.0, FLOOR_Y, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                graphics::Vertex {
                    position: [0.0, FLOOR_Y, (INSTANCED_COLS - 1) as f32 * INSTANCE_SPACING],
                    tex_coords: [0.0, 5.0],
                },
                graphics::Vertex {
                    position: [(INSTANCED_ROWS - 1) as f32 * INSTANCE_SPACING, FLOOR_Y, 0.0],
                    tex_coords: [5.0, 0.0],
                },
                graphics::Vertex {
                    position: [
                        (INSTANCED_ROWS - 1) as f32 * INSTANCE_SPACING,
                        FLOOR_Y,
                        (INSTANCED_COLS - 1) as f32 * INSTANCE_SPACING,
                    ],
                    tex_coords: [5.0, 5.0],
                },
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices_floor"),
            contents: bytemuck::cast_slice(&[
                0u32, 1, 2, 
                1, 3, 2, 
                1, 0, 2, 
                3, 1, 2
            ]),
            usage: wgpu::BufferUsages::INDEX,
        }),
        model_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("model_floor"),
            contents: bytemuck::cast_slice(&[super::graphics::RawMatrix {
                mat: Matrix4::identity().into(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        is_instanced_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("is_instanced_floor"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        num_indices: 12,
        instances_buffer: None,
        num_instances: None,
        shown_instances: None,
    }
}

fn build_sphere(device: &wgpu::Device, instances: &Vec<Instance>) -> RenderObject {
    let (vertices, indices) = pythagoras_sphere(5.0, 75);

    RenderObject {
        vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertices_sphere"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices_sphere"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        }),
        model_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("model_sphere"),
            contents: bytemuck::cast_slice(&[super::graphics::RawMatrix {
                mat: Matrix4::identity().into(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        is_instanced_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("is_instanced_sphere"),
            contents: bytemuck::cast_slice(&[1u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        num_indices: indices.len() as u32,
        instances_buffer: Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("sphere_instance_buffer"),
                contents: bytemuck::cast_slice(
                    &instances.iter().map(Instance::as_raw).collect::<Vec<_>>(),
                ),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        ),
        num_instances: Some(instances.len() as u32),
        shown_instances: Some(instances.len() as u32),
    }
}

fn pythagoras_sphere(radius: f64, lod: u32) -> (Vec<graphics::Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let factor = radius / lod as f64;
    let mut verts_per_band = 0;
    let mut bands = 0;

    let mut y = -radius;
    for _ in 0..=(lod * 2) {
        verts_per_band = 0;
        let band_radius = (radius * radius - y * y).max(0.0).sqrt();
        let band_factor = band_radius / lod as f64;

        let mut x = -band_radius;
        for _ in 0..=(lod * 2) {
            let z = (band_radius * band_radius - x * x).max(0.0).sqrt();
            let tex = [((x / radius) as f32).abs(), ((z / radius) as f32).abs()];
            vertices.push(graphics::Vertex {
                position: [x as f32, y as f32, z as f32],
                tex_coords: tex,
            });
            vertices.push(graphics::Vertex {
                position: [x as f32, y as f32, -z as f32],
                tex_coords: tex,
            });

            x += band_factor;
            verts_per_band += 2;
        }

        y += factor;
        bands += 1;
    }

    let mut indices = Vec::new();
    for i in 0..bands - 1 {
        for j in 0..verts_per_band - 2 {
            if j % 2 != 0 {
                indices.push((i * verts_per_band) + j);
                indices.push(((i + 1) * verts_per_band) + j);
                indices.push(((i + 1) * verts_per_band) + j + 2);
                indices.push((i * verts_per_band) + j);
                indices.push(((i + 1) * verts_per_band) + j + 2);
                indices.push((i * verts_per_band) + j + 2);
            } else {
                indices.push((i * verts_per_band) + j);
                indices.push(((i + 1) * verts_per_band) + j + 2);
                indices.push(((i + 1) * verts_per_band) + j);
                indices.push((i * verts_per_band) + j);
                indices.push((i * verts_per_band) + j + 2);
                indices.push(((i + 1) * verts_per_band) + j + 2);
            }
        }
    }

    (vertices, indices)
}