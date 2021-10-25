use crate::{math::{Mat4x4f, Vec3f}, shapes::Shape};

pub struct Entity {
    pub shape: Shape,
    pub local_transform: Mat4x4f,
}

impl Entity {

    pub fn with_geometry(shape: Shape) -> Self {
        Self {
            shape,
            local_transform: Mat4x4f::identity(),
        }
    }

    pub fn translate(&mut self, translation: Vec3f) {
        self.local_transform.m[0][3] += translation.x;
        self.local_transform.m[1][3] += translation.y;
        self.local_transform.m[2][3] += translation.z;
    }

    pub fn set_translation(&mut self, translation: Vec3f) {
        self.local_transform.m[0][3] = translation.x;
        self.local_transform.m[1][3] = translation.y;
        self.local_transform.m[2][3] = translation.z;
    }

    pub fn get_translation(&self) -> Vec3f {
        let m = &self.local_transform.m;
        Vec3f::new(m[0][3], m[1][3], m[2][3])
    }

    pub fn set_rotation_y(&mut self, theta: f32) {
        let (costheta, sintheta) = theta.sin_cos();
        self.local_transform.m[0][0] = costheta;
        self.local_transform.m[2][0] = -sintheta;
        self.local_transform.m[0][2] = sintheta;
        self.local_transform.m[2][2] = costheta;
    }
}
