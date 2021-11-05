use std::{ops::{AddAssign, Mul}, path::Path};

pub struct Texture {
    data: Vec<Color>,
    width: usize,
    height: usize,
}

impl Texture {
    fn new(data: Vec<Color>, width: usize, height: usize) -> Self {
        Texture { data, width, height }
    }

    pub fn load_from_file<P: AsRef<Path>>(filepath: P) -> Self {
        let texture = lodepng::decode32_file(filepath).unwrap();
        let (w, h) = (texture.width, texture.height);
        let mut data = Vec::with_capacity(w * h);
        for pixel in &texture.buffer {
            data.push(Color::new(pixel.r, pixel.g, pixel.b)); 
        }
        Self::new(data, w, h)
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let x = (u * (self.width - 1) as f32) as usize;
        let y = (v * (self.height - 1) as f32) as usize;
        self.data[y * self.width + x]
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {

    pub const WHITE: Self = Color::new(255, 255, 255);
    pub const BLACK: Self = Color::new(0  , 0  , 0  );
    pub const GRAY:  Self = Color::new(128, 128, 128);
    pub const RED:   Self = Color::new(255, 0  , 0  );
    pub const GREEN: Self = Color::new(0  , 255, 0  );
    pub const BLUE:  Self = Color::new(0  , 0  , 255);    

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub const fn is_not_black(&self) -> bool {
        !(self.r == 0 && self.g == 0 && self.b == 0)
    }
}

impl Mul<f32> for Color {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        let rhs = if rhs < 0.0 { 0.0 } else if rhs > 1.0 { 1.0 } else { rhs };
        let new_color = Self::new(
            (self.r as f32 * rhs) as u8, 
            (self.g as f32 * rhs) as u8, 
            (self.b as f32 * rhs) as u8,
        );
        if self.is_not_black() && !new_color.is_not_black() {
            Color::new(1, 1, 1)
        } else {
            new_color
        }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}