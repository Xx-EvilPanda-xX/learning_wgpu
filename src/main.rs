
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder
};

pub mod graphics;
mod app;

fn main() {
    run_app();
}

fn run_app() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_inner_size(winit::dpi::PhysicalSize::new(1280, 720)).build(&event_loop).unwrap();
    
    let mut app = app::App::new(&window);

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
            app.input_state.toggle_cooldown -= 0.005;
        }
        _ => {}
    });
}