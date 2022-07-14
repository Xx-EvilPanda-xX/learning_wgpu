use winit::event::{KeyboardInput, VirtualKeyCode, ElementState};

pub struct InputState {
    pub space_pressed: bool,
    pub shift_pressed: bool,
    pub forward_pressed: bool,
    pub backward_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    unhandled_mouse_move: (f64, f64),
}

impl InputState {
    const SPACE: VirtualKeyCode = VirtualKeyCode::Space;
    const SHIFT: VirtualKeyCode = VirtualKeyCode::LShift;
    const FORWARD: VirtualKeyCode = VirtualKeyCode::W;
    const BACK: VirtualKeyCode = VirtualKeyCode::S;
    const LEFT: VirtualKeyCode = VirtualKeyCode::A;
    const RIGHT: VirtualKeyCode = VirtualKeyCode::D;

    pub fn new() -> Self {
        InputState {
            space_pressed: false,
            shift_pressed: false,
            forward_pressed: false,
            backward_pressed: false,
            left_pressed: false,
            right_pressed: false,
            unhandled_mouse_move: (0.0, 0.0)
        }
    }

    pub fn update_keyboard(&mut self, input: &KeyboardInput) {
        match input {
            KeyboardInput { 
                state,
                virtual_keycode,
                ..
            } => {
                if let Some(key) = virtual_keycode {
                    match *key {
                        InputState::SPACE => self.space_pressed = if let ElementState::Pressed = state { true } else { false },
                        InputState::SHIFT => self.shift_pressed = if let ElementState::Pressed = state { true } else { false },
                        InputState::FORWARD => self.forward_pressed = if let ElementState::Pressed = state { true } else { false },
                        InputState::BACK => self.backward_pressed = if let ElementState::Pressed = state { true } else { false },
                        InputState::LEFT => self.left_pressed = if let ElementState::Pressed = state { true } else { false },
                        InputState::RIGHT => self.right_pressed = if let ElementState::Pressed = state { true } else { false },
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn update_mouse(&mut self, delta: &(f64, f64)) {
        self.unhandled_mouse_move = *delta;
        self.unhandled_mouse_move.0 /= 2500.0;
        self.unhandled_mouse_move.1 /= 2500.0;
    }

    pub fn handle_mouse_move(&mut self) -> (f64, f64) {
        let unhandled = self.unhandled_mouse_move;
        self.unhandled_mouse_move = (0.0, 0.0);
        unhandled
    }
}