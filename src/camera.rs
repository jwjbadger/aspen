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

/// Represents a generic camera that can be used in [`App`]
///
/// [`App`]: crate::App
pub trait Camera {
    /// Called whenever the app is resized
    fn resize(&mut self, width: f32, height: f32);
    /// Required to work with the WGPU renderer. Generates a view_projection matrix to translate
    /// objects into clip space.
    fn build_view_projection_matrix(&self) -> nalgebra::Matrix4<f32>;
}

/// A generic example of a fly camera.
#[derive(Debug, Clone)]
pub struct FlyCamera {
    /// The location of the viewer.
    pub eye: nalgebra::Point3<f32>,
    /// The direction the viewer is looking (normalized).
    pub dir: nalgebra::Vector3<f32>,
    /// Points straight up from the viewer to determine how the camera is rotated.
    pub up: nalgebra::Vector3<f32>,
    /// The aspect ratio of the screen.
    pub aspect: f32,
    /// The fov of the screen.
    pub fovy: f32,
    /// Anything closer than this will be clipped.
    pub znear: f32,
    /// Anything further than this will be clipped.
    pub zfar: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            eye: nalgebra::Point3::new(1.0, 1.0, 0.0),
            dir: (nalgebra::Point3::origin() - nalgebra::Point3::new(1.0, 1.0, 0.0)).normalize(),
            up: nalgebra::Vector3::y(),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl Camera for FlyCamera {
    fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    fn build_view_projection_matrix(&self) -> nalgebra::Matrix4<f32> {
        let view = nalgebra::Matrix4::look_at_rh(&self.eye, &(self.eye + self.dir), &self.up);
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

impl FlyCamera {
    /// Turns the camera in a specific direction offset from its current rotation.
    pub fn turn(&mut self, angle: nalgebra::UnitQuaternion<f32>) {
        self.dir = angle.transform_vector(&self.dir);
        self.up = angle.transform_vector(&self.up);
    }
}
