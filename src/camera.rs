use bytemuck::NoUninit;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: nalgebra::Matrix4<f32> = nalgebra::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, NoUninit)]
pub(crate) struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_proj: nalgebra::Matrix4::identity().into(),
        }
    }

    pub(crate) fn update(&mut self, camera: &impl Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }

    pub(crate) fn update_raw(&mut self, view_proj: nalgebra::Matrix4<f32>) {
        self.view_proj = view_proj.into();
    }
}

pub trait Camera {
    fn resize(&mut self, width: f32, height: f32);
    fn build_view_projection_matrix(&self) -> nalgebra::Matrix4<f32>;
}

pub struct FpvCamera {
    pub eye: nalgebra::Point3<f32>,
    pub target: nalgebra::Point3<f32>,
    pub up: nalgebra::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for FpvCamera {
    fn default() -> Self {
        Self {
            eye: nalgebra::Point3::new(1.0, 1.0, 0.0),
            target: nalgebra::Point3::new(0.0, 0.0, 0.0),
            up: nalgebra::Vector3::y(),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl Camera for FpvCamera {
    fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    fn build_view_projection_matrix(&self) -> nalgebra::Matrix4<f32> {
        let view = nalgebra::Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let projection = nalgebra::Perspective3::new(
            self.aspect,
            self.fovy * std::f32::consts::PI / 180.0,
            self.znear,
            self.zfar,
        )
        .to_homogeneous();
        OPENGL_TO_WGPU_MATRIX * projection * view
    }
}
