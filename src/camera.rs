use cgmath::{Vector3, Point3, InnerSpace};

#[derive(Debug)]
pub struct Camera {
    pub loc: cgmath::Point3<f32>,
    forward: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,
    right: cgmath::Vector3<f32>,
    yaw: f32,
    pitch: f32,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    sens: f32,
    speed: f32
}

pub const GL_TO_WGPU: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
);

impl Camera {
    const WORLD_UP: Vector3<f32> = Vector3 { x: 0.0, y: 1.0, z: 0.0 };

    pub fn new(loc: Point3<f32>, yaw: f32, pitch: f32, aspect: f32, fovy: f32, znear: f32, zfar: f32, sens: f32, speed: f32) -> Self {
        let mut cam = Camera {
            loc,
            forward: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 0.0, 0.0),
            right: Vector3::new(0.0, 0.0, 0.0),
            yaw,
            pitch,
            aspect,
            fovy,
            znear,
            zfar,
            sens,
            speed
        };
        cam.calc_vecs();
        cam
    }

    pub fn build_view_proj(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.loc, self.loc + self.forward, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        GL_TO_WGPU * proj * view
    }

    pub fn update_pos(&mut self, movement: Vector3<f32>, dt: f32) {
        let s = &self.speed;
        let m = &movement;

        let x = Vector3 { x: self.forward.x, y: 0.0, z: self.forward.z }.normalize();
        self.loc.x += s * x.x * m.x * dt;
        self.loc.z += s * x.z * m.x * dt;

        self.loc.x += s * self.right.x * m.z * dt;
        self.loc.y += s * self.right.y * m.z * dt;
        self.loc.z += s * self.right.z * m.z * dt;

        self.loc.y += s * movement.y * dt;

        self.calc_vecs();
    }
    
    pub fn update_look(&mut self, look: (f32, f32)) {
        self.yaw += self.sens * look.0;
        self.pitch += self.sens * -look.1;

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

    fn calc_vecs(&mut self) {
        let forward = Vector3 { x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
                                            y: self.pitch.to_radians().sin(),
                                            z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos() };
        
        self.forward = forward.normalize();
        self.right = forward.cross(Camera::WORLD_UP).normalize();
        self.up = self.right.cross(forward).normalize();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4]
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        CameraUniform { view_proj: cgmath::Matrix4::identity().into() }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_proj().into();
    }
}