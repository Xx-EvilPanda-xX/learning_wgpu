use cgmath::{InnerSpace, Point3, Vector3, Matrix4, Vector2};

use crate::input;
use crate::app::INSTANCED_ROWS;
use crate::app::INSTANCED_COLS;
use crate::app::INSTANCE_SPACING;

#[derive(Debug)]
pub struct Camera {
    pub loc: Point3<f32>,
    pub vel: Vector3<f32>,
    pub acc: Vector3<f32>,
    forward: Vector3<f32>,
    up: Vector3<f32>,
    right: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    aspect: f32,
    speed: f32,
}

pub const GL_TO_WGPU: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    const WORLD_UP: Vector3<f32> = Vector3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    const SPRINT_SPEED: f32 = 10.0;
    const WALK_SPEED: f32 = 5.0;
    const DEACCELERATION: f32 = 5.0;
    const ACCELERATION: f32 = 5.0;
    const BORDER_SPACE: f32 = 150.0;
    const MAX_POS: Vector3<f32> = Vector3 {
        x: INSTANCED_ROWS as f32 * INSTANCE_SPACING + Self::BORDER_SPACE,
        y: 100.0,
        z: INSTANCED_COLS as f32 * INSTANCE_SPACING + Self::BORDER_SPACE
    };
    const MIN_POS: Vector3<f32> = Vector3 { x: -Self::BORDER_SPACE, y: -Self::BORDER_SPACE, z: -Self::BORDER_SPACE };
    const FOVY: f32 = 90.0;
    const ZNEAR: f32 = 0.1;
    const ZFAR: f32 = 1000.0;
    const SENS: f32 = 20.0;

    pub fn new(
        loc: Point3<f32>,
        yaw: f32,
        pitch: f32,
        aspect: f32,
    ) -> Self {
        let mut cam = Camera {
            loc,
            vel: Vector3::new(0.0, 0.0, 0.0),
            acc: Vector3::new(0.0, 0.0, 0.0),
            forward: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 0.0, 0.0),
            right: Vector3::new(0.0, 0.0, 0.0),
            yaw,
            pitch,
            aspect,
            speed: Self::WALK_SPEED,
        };
        cam.calc_vecs();
        cam
    }

    pub fn build_view_proj(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.loc, self.loc + self.forward, self.up);
        let proj = cgmath::perspective(cgmath::Deg(Self::FOVY), self.aspect, Self::ZNEAR, Self::ZFAR);
        GL_TO_WGPU * proj * view
    }

    pub fn update_pos(&mut self, dt: f32, input: &input::InputState) {
        self.update_acc(input);
        self.update_vel(dt);
        self.update_speed(dt, input);
        self.update_loc(dt);

        if self.loc.x > Self::MAX_POS.x {
            self.loc.x = Self::MAX_POS.x;
            self.vel.x = -self.vel.x;
        }
        if self.loc.y > Self::MAX_POS.y {
            self.loc.y = Self::MAX_POS.y;
            self.vel.y = -self.vel.y;
        }
        if self.loc.z > Self::MAX_POS.z {
            self.loc.z = Self::MAX_POS.z;
            self.vel.z = -self.vel.z;
        }
        if self.loc.x < Self::MIN_POS.x {
            self.loc.x = Self::MIN_POS.x;
            self.vel.x = -self.vel.x;
        }
        if self.loc.y < Self::MIN_POS.y {
            self.loc.y = Self::MIN_POS.y;
            self.vel.y = -self.vel.y;
        }
        if self.loc.z < Self::MIN_POS.z {
            self.loc.z = Self::MIN_POS.z;
            self.vel.z = -self.vel.z;
        }
    }

    fn update_loc(&mut self, dt: f32) {
        let s = self.speed;
        let v = &self.vel;

        self.loc.x += s * v.x * dt;
        self.loc.y += s * v.y * dt;
        self.loc.z += s * v.z * dt;
    }

    fn update_speed(&mut self, dt: f32, input: &input::InputState) {
        if input.ctrl_pressed && input.movement_key_pressed() {
            self.speed += dt * 5.0;
        } else {
            self.speed -= dt * 5.0;
        }

        if self.speed > Self::SPRINT_SPEED {
            self.speed = Self::SPRINT_SPEED;
        }
        if self.speed < Self::WALK_SPEED {
            self.speed = Self::WALK_SPEED;
        }
    }

    fn update_vel(&mut self, dt: f32) {
        let forward = Vector3::new(self.forward.x, 0.0, self.forward.z).normalize();
        let right = Vector3::new(self.right.x, 0.0, self.right.z).normalize();

        self.vel.x += self.acc.x * forward.x * dt;
        self.vel.z += self.acc.x * forward.z * dt;

        self.vel.x += self.acc.z * right.x * dt;
        self.vel.z += self.acc.z * right.z * dt;

        self.vel.y += self.acc.y * dt;

        let amp = dt * Self::DEACCELERATION;
        let vel_2d = Vector2::new(self.vel.x, self.vel.z);
        const RIGHT_ANGLE: f32 = std::f32::consts::PI / 2.0;

        // when not accelerating in x, try to deaccelerate that vel component.
        // done by nudging the velocity towards the right vector using the forward vector
        if self.acc.x == 0.0 && self.acc.z != 0.0 {
            let forward_2d = Vector2::new(forward.x, forward.z);
            // calculate the angle between the velocity vector and forward vector (used to determine whether to add or sub from vel)
            let theta_right_vel = (forward_2d.dot(vel_2d) / (forward_2d.magnitude() * vel_2d.magnitude())).acos();
            if theta_right_vel > RIGHT_ANGLE {
                // nudge velocity
                self.vel.x += forward.x * amp;
                self.vel.z += forward.z * amp;
            } else {
                // nudge velocity
                self.vel.x -= forward.x * amp;
                self.vel.z -= forward.z * amp;
            }
        // repeat for when not accelerating on the z
        } else if self.acc.x != 0.0 && self.acc.z == 0.0 { 
            let right_2d = Vector2::new(right.x, right.z);
            let theta_right_vel = (right_2d.dot(vel_2d) / (right_2d.magnitude() * vel_2d.magnitude())).acos();
            if theta_right_vel > RIGHT_ANGLE {
                self.vel.x += right.x * amp;
                self.vel.z += right.z * amp;
            } else {
                self.vel.x -= right.x * amp;
                self.vel.z -= right.z * amp;
            }
        // deaccelerate both x and z when neither are accelerating
        } else if self.acc.x == 0.0 && self.acc.z == 0.0 && vel_2d.x != 0.0 && vel_2d.y != 0.0 {
            let decreased = vel_2d.normalize_to((vel_2d.magnitude() - amp).max(0.0));
            self.vel.x = decreased.x;
            self.vel.z = decreased.y;
        }

        // deaccelerate y
        if self.acc.y == 0.0 {
            step(&mut self.vel.y, 0.0, amp);
        }
    }

    fn update_acc(&mut self, input: &input::InputState) {
        self.acc = Vector3::new(0.0, 0.0, 0.0);
        let acc = Self::ACCELERATION + Self::DEACCELERATION;
        if input.forward_pressed {
            self.acc.x += acc;
        }
        if input.backward_pressed {
            self.acc.x -= acc;
        }
        if input.right_pressed {
            self.acc.z += acc;
        }
        if input.left_pressed {
            self.acc.z -= acc;
        }
        if input.space_pressed {
            self.acc.y += acc;
        }
        if input.shift_pressed {
            self.acc.y -= acc;
        }
    }

    pub fn update_look(&mut self, look: (f32, f32), dt: f32) {
        self.yaw += Self::SENS * look.0 * dt;
        self.pitch += Self::SENS * -look.1 * dt;

        if self.yaw > 360.0 {
            self.yaw = 0.0;
        }
        if self.yaw < 0.0 {
            self.yaw = 360.0;
        }
        if self.pitch > 89.99 {
            self.pitch = 89.99;
        }
        if self.pitch < -89.99 {
            self.pitch = -89.99;
        }

        self.calc_vecs();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    fn calc_vecs(&mut self) {
        let forward = Vector3 {
            x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            y: self.pitch.to_radians().sin(),
            z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        };

        self.forward = forward.normalize();
        self.right = forward.cross(Camera::WORLD_UP).normalize();
        self.up = self.right.cross(forward).normalize();
    }
}

fn step(x: &mut f32, to: f32, amp: f32) {
    if *x < to {
        *x += amp;
        if *x > to {
            *x = to;
        }
    } else {
        *x -= amp;
        if *x < to {
            *x = to;
        }
    }
}