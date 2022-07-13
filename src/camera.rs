pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32
}

pub const GL_TO_WGPU: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
);

impl Camera {
    pub fn new(eye: cgmath::Point3<f32>, target: cgmath::Point3<f32>, up: cgmath::Vector3<f32>,
            aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Camera {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar
        }
    }

    pub fn build_view_proj(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        GL_TO_WGPU * proj * view
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