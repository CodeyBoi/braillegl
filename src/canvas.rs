use std::{process::Command, str::from_utf8};

use termion::{clear, color::{Color, Rgb}, cursor, terminal_size};

use crate::{entity::Entity, math::{Mat4x4f, Vec3f}};

pub struct Canvas {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
    win_x: i32,
    win_y: i32,
    pix_w: i32,
    pix_h: i32,
    projection_matrix: Mat4x4f,
    camera: Camera,
}

impl Canvas {
    pub fn pix2cell(&mut self, (x, y): (i32, i32)) -> (i32, i32) {
        let x = (x - self.win_x) * self.width as i32 / self.pix_w;
        let y = (y - self.win_y) * self.height as i32 / self.pix_h;
        (x, y)
    }

    pub fn set(&mut self, x: i32, y: i32, brightness: u8) {
        if !(x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32) {
            return;
        }
        self.pixels[(y * self.width as i32 + x) as usize] = brightness;
    }

    pub fn draw_entity(&mut self, e: &Entity) {
        for (tri, normal) in e.shape.indices().zip(&e.shape.normals) {
            let p0 = e.shape.get(tri.0).position;
            let p1 = e.shape.get(tri.1).position;
            let p2 = e.shape.get(tri.2).position;

            // Apply local transform
            let mut tp0 = e.local_transform.mul(&p0, true);
            let mut tp1 = e.local_transform.mul(&p1, true);
            let mut tp2 = e.local_transform.mul(&p2, true);
            let n = e.local_transform.mul(&normal, false);

            if n.dot(&(tp0 - self.camera.position)) > 0.0 {
                // Cull back faces
                continue;
            }

            let light_direction = Vec3f::new(1.0, -1.0, -1.0).normalize();

            // Project into a 2x2x2 box
            tp0 = self.projection_matrix.mul(&tp0, true);
            tp1 = self.projection_matrix.mul(&tp1, true);
            tp2 = self.projection_matrix.mul(&tp2, true);

            // All values are in the interval [-1, 1]
            tp0.x = (tp0.x + 1.0) * self.width as f32 / 2.0;
            tp0.y = (tp0.y + 1.0) * self.height as f32 / 2.0;
            tp1.x = (tp1.x + 1.0) * self.width as f32 / 2.0;
            tp1.y = (tp1.y + 1.0) * self.height as f32 / 2.0;
            tp2.x = (tp2.x + 1.0) * self.width as f32 / 2.0;
            tp2.y = (tp2.y + 1.0) * self.height as f32 / 2.0;


            let brightness = -n.dot(&light_direction);
            let brightness = if brightness > 0.0 {
                brightness
            } else {
                0.01
            };
            self.fill_triangle(
                tp0.x as i32, tp0.y as i32, 
                tp1.x as i32, tp1.y as i32, 
                tp2.x as i32, tp2.y as i32,
                (brightness * 255.0) as u8,
            );
        }
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, brightness: u8) {
        self.set(x0, y0, 255);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let steps = dx.abs().max(dy.abs());
        if steps == 0 {
            return;
        }
        if dx.abs() > dy.abs() {
            let x_dir = dx / dx.abs();
            let mut current_x = x0;
            let mut current_y = y0 as f32 + 0.5;
            let dy = dy as f32 / steps as f32;
            for _ in 0..steps {
                current_x += x_dir;
                current_y += dy;
                self.set(current_x, current_y as i32, brightness);
            }
        } else {
            let y_dir = dy / dy.abs();
            let mut current_x = x0 as f32 + 0.5;
            let mut current_y = y0;
            let dx = dx as f32 / steps as f32;
            for _ in 0..steps {
                current_x += dx;
                current_y += y_dir;
                self.set(current_x as i32, current_y, brightness);
            }
        }
    }

    pub fn draw_triangle(&mut self, 
        x0: i32, y0: i32, 
        x1: i32, y1: i32,
        x2: i32, y2: i32,
        brightness: u8) 
    {
        self.draw_line(x0, y0, x1, y1, brightness);
        self.draw_line(x1, y1, x2, y2, brightness);
        self.draw_line(x2, y2, x0, y0, brightness);
    }

    pub fn fill_triangle(&mut self, 
        x0: i32, y0: i32, 
        x1: i32, y1: i32,
        x2: i32, y2: i32,
        brightness: u8) 
    {
        // Fill in end points
        self.set(x0, y0, brightness);
        self.set(x1, y1, brightness);
        self.set(x2, y2, brightness);

        // Sort points by y-coord
        let (x0, y0, x1, y1) = if y0 < y1 {
            (x0, y0, x1, y1)
        } else {
            (x1, y1, x0, y0)
        };
        let (x0, y0, x2, y2) = if y0 < y2 {
            (x0, y0, x2, y2)
        } else {
            (x2, y2, x0, y0)
        };
        let (x1, y1, x2, y2) = if y1 < y2 {
            (x1, y1, x2, y2)
        } else {
            (x2, y2, x1, y1)
        };
        // Now (x0, y0) is always lowest
        let mut x012 = Self::interpolate(y0, x0, y1, x1);
        x012.pop();
        x012.append(&mut Self::interpolate(y1, x1, y2, x2));
        let x02 = Self::interpolate(y0, x0, y2, x2);
        let m = x02.len() / 2;
        let (x_left, x_right) = if x02[m] < x012[m] {
            (x02, x012)
        } else {
            (x012, x02)
        };
        for y in y0..y2 + 1 {
            let i = (y - y0) as usize;
            for x in x_left[i]..x_right[i] + 1 {
                self.set(x, y, brightness);
            }
        }
    }

    fn interpolate(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<i32> {
        assert!(x0 <= x1);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let y_step = dy as f32 / dx as f32;
        let mut y = y0 as f32 + 0.5;
        let mut values = Vec::with_capacity(dx as usize);
        values.push(y as i32);
        for _ in 0..dx {
            y += y_step;
            values.push(y as i32);
        }
        values
    }

    pub fn clear(&mut self) {
        self.pixels = vec![0; self.width * self.height];
    }

    /// Computes the resulting image as a string to be printed
    pub fn to_s(&self) -> String {
        const INDEX_OFFSETS: [(usize, usize); 8] = [
            (0, 0), (0, 1), (0, 2),
            (1, 0), (1, 1), (1, 2),
            (0, 3), (1, 3),
        ];
        let mut string = String::with_capacity(self.pixels.len() * 3 / 2 + 4);
        string.push_str(&clear::All.to_string());
        for row in 0..self.height / 4 {
            for col in 0..self.width / 2 {
                let (pix_row, pix_col) = (row * 4, col * 2);
                let mut braille_code = 0x2800;
                let mut brightness: u8 = 0;
                for (i, (dx, dy)) in INDEX_OFFSETS.iter().enumerate() {
                    let p_value = self.pixels[(pix_row + dy) * self.width + pix_col + dx];
                    if  p_value > 0 {
                        braille_code += 2_u32.pow(i as u32);
                        brightness += p_value / 8;
                    }
                }
                if braille_code != 0x2800 {
                    string.push_str(&cursor::Goto(
                        (col as u16).saturating_add(1), 
                        (row as u16).saturating_add(1)).to_string()
                    );
                    string.push_str(&Rgb(brightness, brightness, brightness).fg_string());
                    string.push(char::from_u32(braille_code).unwrap());
                }
            }
        }
        string
    }

    pub fn new() -> Self {
        let wpos = Command::new("sh")
            .arg("-c")
            .arg(r"xdotool getwindowfocus getwindowgeometry --shell | sed /[XYHT]=/P -n | echo -n $(tr -dc '0-9\n')")
            .output()
            .expect("failed when getting window position.")
            .stdout;
        let wpos: Vec<i32> = from_utf8(&wpos).unwrap().split(" ").map(|d|
            d.parse::<i32>().unwrap()
        ).collect();
        let (win_x, win_y, pix_w, pix_h) = (wpos[0], wpos[1], wpos[2] - 13, wpos[3] - 10);

        let (win_x, win_y, pix_w, pix_h) = (0, 0, 1353, 758);

        let (width, height) = terminal_size().unwrap();
        let (width, height) = (width as usize * 2, height as usize * 4);
        let pixels = vec![0; width * height];
        let projection_matrix = Mat4x4f::get_projection(
            9.0 / 16.0, 
            90.0, 
            0.1, 1000.0
        );
        let camera = Camera {
            position: Vec3f::new(0.0, 0.0, 0.0),
            direction: Vec3f::new(0.0, 0.0, 1.0),
        };
        Self { pixels, width, height, win_x, win_y, pix_w, pix_h, projection_matrix, camera }
    }
}

pub struct Camera {
    pub position: Vec3f,
    pub direction: Vec3f,
}