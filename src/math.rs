use std::{f32::consts::PI, ops::{Add, Neg, Sub}};

#[derive(Clone, Copy, Debug)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn normalize(&self) -> Self {
        let l = self.length();
        self.scale(1.0 / l)
    }

    pub fn length(&self) -> f32 {
        (self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.y*rhs.z - self.z*rhs.y, 
            self.z*rhs.x - self.x*rhs.z, 
            self.x*rhs.y - self.y*rhs.x
        )
    }

    pub fn scale(&self, scale: f32) -> Self {
        Self::new(self.x * scale, self.y * scale, self.z * scale)
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        self.x*rhs.x + self.y*rhs.y + self.z*rhs.z
    }
}

impl Add for Vec3f {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Neg for Vec3f {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Sub for Vec3f {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

pub struct Mat4x4f {
    pub m: [[f32; 4]; 4],
}

impl Mat4x4f {
    pub const fn new(
        m11: f32, m12: f32, m13: f32, m14: f32,
        m21: f32, m22: f32, m23: f32, m24: f32,
        m31: f32, m32: f32, m33: f32, m34: f32,
        m41: f32, m42: f32, m43: f32, m44: f32,
    ) -> Self {
        Self {
            m:
            [[m11, m12, m13, m14],
             [m21, m22, m23, m24],
             [m31, m32, m33, m34],
             [m41, m42, m43, m44]],
        }
    }

    pub const fn zero() -> Self {
        Self::new(
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0, 
        )
    }

    pub fn mul(&self, rhs: &Vec3f, translate: bool) -> Vec3f {
        let t = if translate { 1.0 } else { 0.0 };
        let m = self.m;
        let v = Vec3f::new(
            m[0][0]*rhs.x + m[0][1]*rhs.y + m[0][2]*rhs.z + m[0][3]*t,
            m[1][0]*rhs.x + m[1][1]*rhs.y + m[1][2]*rhs.z + m[1][3]*t,
            m[2][0]*rhs.x + m[2][1]*rhs.y + m[2][2]*rhs.z + m[2][3]*t            
        );
        // Remove below later
        let scale = m[3][0]*rhs.x + m[3][1]*rhs.y + m[3][2]*rhs.z + m[3][3]*t;
        if scale != 0.0 {
            v.scale(1.0 / scale)
        } else {
            v
        }
    }

    pub fn matmul(&self, rhs: &Self) -> Self {
        let mut result = Self::zero();
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.m[i][j] += self.m[i][k] * rhs.m[k][j];
                }
            }
        }
        result
    }

    pub fn projection(aspect_ratio: f32, fov: f32, znear: f32, zfar: f32) -> Self {
        let angle = fov * PI / 180.0;
        let f = 1.0 / (angle / 2.0).tan();
        let q = zfar / (zfar - znear);
        Self::new(
            f, 0.0, 0.0, 0.0,
            0.0, aspect_ratio*f, 0.0, 0.0,
            0.0, 0.0, q, -znear*q,
            0.0, 0.0, 1.0, 0.0,
        )
    }

    pub fn rotate_y(theta: f32) -> Self {
        let mut result = Self::identity();
        let (sintheta, costheta) = theta.sin_cos();
        result.m[0][0] = costheta;
        result.m[2][0] = -sintheta;
        result.m[0][2] = sintheta;
        result.m[2][2] = costheta;
        result
    }

    pub const fn identity() -> Self {
        Self::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }
}