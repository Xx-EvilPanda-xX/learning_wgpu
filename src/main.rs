use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use log::warn;

mod app;
mod camera;
pub mod graphics;
mod input;

fn main() {
    run_app();
}

fn run_app() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(1600, 900))
        .with_position(winit::dpi::PhysicalPosition::new(100, 50))
        .with_title("learning_wgpu")
        .build(&event_loop)
        .unwrap();
    window.set_cursor_visible(false);

    let mut app = app::App::new(&window);
    warn!("Size of application on stack: {}", std::mem::size_of::<app::App>());
    let mut last_frame = std::time::Instant::now();
    let mut is_focused = true;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Focused(focused) => {
                is_focused = *focused;
                window.set_cursor_visible(!is_focused);
            }
            _ => {
                app.input(Some(event), None, &window);
            }   
        },
        Event::DeviceEvent { ref event, .. } => {
            if is_focused {
                app.input(None, Some(event), &window);
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            app.update();
            match app.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => app.resize(app.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            let now = std::time::Instant::now();
            app.delta_time = now.duration_since(last_frame).as_secs_f64();
            last_frame = now;
            window.request_redraw();
        }
        _ => {}
    });
}
