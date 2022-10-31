use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Fullscreen},
};
use log::{info, debug};

mod app;
mod camera;
mod graphics;
mod input;

const EXCLUSIVE_FULLSCREEN: bool = false;

fn main() {
    run_app();
}

fn run_app() {
    env_logger::init();
    let event_loop = EventLoop::new();

    info!("Initializing... Please wait.");

    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(1600, 900))
        .with_position(winit::dpi::PhysicalPosition::new(100, 50))
        .with_title("learning_wgpu")
        .with_visible(false)
        .build(&event_loop)
        .expect("Failed to build window");

    info!("Size of application on stack: {}kb", &(std::mem::size_of::<app::App>() as f64 / 1024.0).to_string()[0..4]);
    let mut app = app::App::new(&window);
    let mut last_frame = std::time::Instant::now();
    let mut is_focused = false;
    let mut last_fps_update = std::time::Instant::now();
    let mut frames = 0;
    info!("Done initializing.");

    window.set_visible(true);
    event_loop.run(move |event, window_target, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    match key {
                        VirtualKeyCode::Escape => {
                            if !is_focused {
                                *control_flow = ControlFlow::Exit;
                            } else {
                                is_focused = false;
                                window.set_cursor_visible(true);
                            }
                        }
                        VirtualKeyCode::F11 => {
                            window.set_fullscreen(
                                if let None = window.fullscreen() {
                                    if EXCLUSIVE_FULLSCREEN {
                                        Some(Fullscreen::Exclusive(
                                            window_target
                                                .primary_monitor()
                                                .expect("Failed to get primary monitor")
                                                .video_modes()
                                                .next()
                                                .expect("No fullscreen video modes available")
                                        ))
                                    } else {
                                        Some(Fullscreen::Borderless(None))
                                    }
                                } else {
                                    None
                                }
                            );
                        }
                        _ => app.input(Some(event), None, &window, is_focused)
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    is_focused = true;
                    window.set_cursor_visible(false);
                }
                WindowEvent::Focused(focused) => {
                    is_focused = *focused;
                    window.set_cursor_visible(!is_focused);
                }
                _ => app.input(Some(event), None, &window, is_focused)
            },
            Event::DeviceEvent { ref event, .. } => {
                app.input(None, Some(event), &window, is_focused);
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                app.update();
                match app.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => app.resize(app.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => debug!("SurfaceError: {:?}", e),
                }
            }
            Event::MainEventsCleared => {
                frames += 1;
                let now = std::time::Instant::now();
                if now.duration_since(last_fps_update) >= std::time::Duration::from_secs(1) {
                    window.set_title(&format!("learing_wgpu | FPS: {}", frames));
                    frames = 0;
                    last_fps_update = now;
                }

                let now = std::time::Instant::now();
                app.delta_time = now.duration_since(last_frame).as_secs_f64();
                last_frame = now;
                window.request_redraw();
            }
            _ => {}
        }
    });
}
