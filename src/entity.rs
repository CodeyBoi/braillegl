use std::path::Path;

use crate::{math::{Mat4x4f, Vec3f}, shapes::Shape, texture::{Color, Texture}};

pub struct Entity {
    pub shape: Shape,
    translation: Vec3f,
    direction: Vec3f,
    scale: f32,
    texture: Option<Texture>,
}

impl Entity {
    pub fn with_geometry(shape: Shape) -> Self {
        Self {
            shape,
            translation: Vec3f::zero(),
            direction: Vec3f::new(0.0, 0.0, 1.0),
            scale: 1.0,
            texture: None,
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.translation += Vec3f::new(dx, dy, dz);
    }

    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
        self.translation = Vec3f::new(x, y, z);
    }

    pub fn get_translation(&self) -> Vec3f {
        self.translation
    }

    pub fn set_direction(&mut self, x: f32, y: f32, z: f32) {
        self.direction = Vec3f::new(x, y, z).normalize();
    }

    pub fn get_direction(&self) -> Vec3f {
        self.direction
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn scale(&mut self, scale: f32) {
        self.scale *= scale;
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, filepath: P) {
        self.texture = Some(Texture::load_from_file(filepath));
    }

    /// Samples the entities texture using the texcoords in the
    /// interval [0, 1]. Returns Color(255, 255, 255) if entity
    /// has no texture (if load_texture hasn't been called).
    pub fn sample_texture(&self, (u, v): (f32, f32)) -> Color {
        if let Some(tex) = &self.texture {
            tex.sample(u, v)
        } else {
            // Returns white if there is no texture
            Color::WHITE
        }
    }

    pub fn gen_local_transform(&self) -> Mat4x4f {
        // We are assuming that the direction vector is normalized
        let theta = if self.direction.x >= 0.0 {
            self.direction.z.acos()
        } else {
            -self.direction.z.acos() // TODO fix this shit
        };
        let phi = -self.direction.y.asin();
        let s = Mat4x4f::identity() * self.scale;
        let ry = Mat4x4f::rotate_y(theta);
        let rx = Mat4x4f::rotate_x(phi);
        let mut transform = ry * rx * s;
        transform.m[0][3] = self.translation.x;
        transform.m[1][3] = self.translation.y;
        transform.m[2][3] = self.translation.z;
        transform
    }
}
