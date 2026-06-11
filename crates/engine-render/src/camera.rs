use glam::{Mat4, Vec3};

pub const DEFAULT_FOV_Y: f32 = 70.0_f32.to_radians();

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 8.0),
            yaw: 0.0,
            pitch: -0.35,
            fov_y: DEFAULT_FOV_Y,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 500.0,
        }
    }
}

impl Camera {
    pub fn forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(sy * cp, cy * cp, sp).normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Z).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward(), Vec3::Z)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}
