use std::{fmt::Write};

use termion::{clear, color::Rgb, cursor, terminal_size};

use crate::{entity::Entity, math::{Mat4x4f, Vec3f}, texture::Color};

pub struct Canvas {
    pixels: Vec<Option<Color>>,
    width: usize,
    height: usize,
    win_x: i32,
    win_y: i32,
    pix_w: i32,
    pix_h: i32,
    projection_matrix: Mat4x4f,
    camera: Camera,
    depth_buffer: Vec<f32>,
}

impl Canvas {
    pub fn pix2cell(&mut self, (x, y): (i32, i32)) -> (i32, i32) {
        let x = (x - self.win_x) * self.width as i32 / self.pix_w;
        let y = (y - self.win_y) * self.height as i32 / self.pix_h;
        (x, y)
    }

    pub fn set(&mut self, x: i32, y: i32, color: Color, depth: f32) {
        if !(x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32) {
            return;
        }
        let index = (y * self.width as i32 + x) as usize;
        if self.depth_buffer[index] > depth {
            return;
        }
        self.pixels[index] = Some(color);
        self.depth_buffer[index] = depth;
    }

    pub fn draw_entity(&mut self, e: &Entity) {
        
        let light_direction = Vec3f::new(1.0, -1.0, -1.0).normalize();

        for tri in e.shape.triangles() {

            // Get vertices
            let v0 = e.shape.get(tri.0);
            let v1 = e.shape.get(tri.1);
            let v2 = e.shape.get(tri.2);            

            // Apply local transform
            let lt = e.gen_local_transform();
            let tp0 = lt.vecmul(&v0.position, true);
            let tp1 = lt.vecmul(&v1.position, true);
            let tp2 = lt.vecmul(&v2.position, true);

            // Cull back faces
            let face_normal = (tp1 - tp0).cross(&(tp2 - tp0)).normalize();
            if face_normal.dot(&(self.camera.position - tp0)) < 0.0 {
                continue;
            }

            // This is wrong. TODO: Transform normals with (M^-1)^T instead
            // Might explain the visual artifacts
            // let n0 = lt.vecmul(&v0.normal, false).normalize();
            // let n1 = lt.vecmul(&v1.normal, false).normalize();
            // let n2 = lt.vecmul(&v2.normal, false).normalize();

            // Project into a 2x2x2 box
            let mut tp0 = self.projection_matrix.vecmul(&tp0, true);
            let mut tp1 = self.projection_matrix.vecmul(&tp1, true);
            let mut tp2 = self.projection_matrix.vecmul(&tp2, true);

            // All values are in the interval [-1, 1]
            tp0.x = (tp0.x + 1.0) * self.width as f32 / 2.0;
            tp0.y = (tp0.y + 1.0) * self.height as f32 / 2.0;
            tp1.x = (tp1.x + 1.0) * self.width as f32 / 2.0;
            tp1.y = (tp1.y + 1.0) * self.height as f32 / 2.0;
            tp2.x = (tp2.x + 1.0) * self.width as f32 / 2.0;
            tp2.y = (tp2.y + 1.0) * self.height as f32 / 2.0;

            let depth = (tp0.z + tp1.z + tp2.z) / 3.0;

            let brightness = (-face_normal.dot(&light_direction)).clamp(0.0, 1.0);

            // Sample texture colors, will be white if texcoords are
            // not defined
            let c0 = if let Some(tc) = v0.texcoord {
                e.sample_texture(tc)
            } else {
                Color::WHITE
            };
            // let c1 = if let Some(tc) = v1.texcoord {
                // e.sample_texture(tc)
            // } else {
                // Color::WHITE
            // };
            // let c2 = if let Some(tc) = v2.texcoord {
                // e.sample_texture(tc)
            // } else {
                // Color::WHITE
            // };
            
            self.fill_triangle(
                tp0.x as i32, tp0.y as i32, 
                tp1.x as i32, tp1.y as i32, 
                tp2.x as i32, tp2.y as i32,
                c0 * brightness, depth
            );
        }
    }

    pub fn draw_line(&mut self, 
        x0: i32, y0: i32, 
        x1: i32, y1: i32, 
        color: Color, depth: f32) 
    {
        self.set(x0, y0, color, depth);
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
                self.set(current_x, current_y as i32, color, depth);
            }
        } else {
            let y_dir = dy / dy.abs();
            let mut current_x = x0 as f32 + 0.5;
            let mut current_y = y0;
            let dx = dx as f32 / steps as f32;
            for _ in 0..steps {
                current_x += dx;
                current_y += y_dir;
                self.set(current_x as i32, current_y, color, depth);
            }
        }
    }

    pub fn draw_triangle(&mut self, 
        x0: i32, y0: i32, 
        x1: i32, y1: i32,
        x2: i32, y2: i32,
        color: Color, depth: f32) 
    {
        self.draw_line(x0, y0, x1, y1, color, depth);
        self.draw_line(x1, y1, x2, y2, color, depth);
        self.draw_line(x2, y2, x0, y0, color, depth);
    }

    pub fn fill_triangle(&mut self, 
        x0: i32, y0: i32, 
        x1: i32, y1: i32,
        x2: i32, y2: i32,
        color: Color, depth: f32)
    {
        // Fill in end points
        self.set(x0, y0, color, depth);
        self.set(x1, y1, color, depth);
        self.set(x2, y2, color, depth);

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
        let dy = (y2 - y0 + 1) as usize;

        let mut x012 = Vec::with_capacity(dy);
        // println!("y0={}, y1={}, y2={}\r", y0, y1, y2);
        for y in y0..y1 {
            x012.push(plerp(y0, x0, y1, x1, y));
        }
        for y in y1..y2 + 1 {
            x012.push(plerp(y1, x1, y2, x2, y));
        }

        let mut x02 = Vec::with_capacity(dy);
        for y in y0..y2 + 1 {
            // println!("y={}, plerp={}\r", y, plerp(y0, x0, y2, x2, y));
            x02.push(plerp(y0, x0, y2, x2, y));
        }

        let m = x02.len() / 2;
        let (x_left, x_right) = if x02[m] < x012[m] {
            (x02, x012)
        } else {
            (x012, x02)
        };
        for y in y0..y2 + 1 {
            let i = (y - y0) as usize;
            for x in x_left[i]..x_right[i] + 1 {
                self.set(x, y, color, depth);
            }
        }

        /// Pixel LERP. If you have a line though two points (x0, y0)
        /// and (x1, y1) then plerp() computes which y-value you have 
        /// at x = `x`.
        /// 
        /// # Arguments
        /// `x0, y0, x1, y1` - the line over which to interpolate.
        /// 
        /// `x` - the x-value for which to compute y.
        fn plerp(x0: i32, y0: i32, x1: i32, y1: i32, x: i32) -> i32 {
            let (dx, dy) = (x1 - x0, y1 - y0);
            if x == x0 || dy == 0 {
                return y0;
            } else if x == x1 {
                return y1;
            }
            let y_step = dy as f32 / dx as f32;
            (y0 as f32 + y_step * (x - x0) as f32 + 0.5) as i32
        }
    }

    pub fn clear(&mut self) {
        let pixs = self.width * self.height;
        self.pixels = vec![None; pixs];
        self.depth_buffer = vec![f32::MIN; pixs];
    }

    /// Computes the resulting image as a string to be printed
    pub fn to_s(&self) -> String {
        const INDEX_OFFSETS: [(usize, usize); 8] = [
            (0, 0), (0, 1), (0, 2),
            (1, 0), (1, 1), (1, 2),
            (0, 3), (1, 3),
        ];
        let mut string = String::with_capacity(self.pixels.len() * 3 / 2 + 4);
        string.write_str(&clear::All.to_string()).unwrap();
        for row in 0..self.height / 4 {
            for col in 0..self.width / 2 {
                let (pix_row, pix_col) = (row * 4, col * 2);
                let mut braille_code = 0x2800;
                let mut cel_color = Color::BLACK;
                for (i, (dx, dy)) in INDEX_OFFSETS.iter().enumerate() {
                    let index = (pix_row + dy) * self.width + pix_col + dx;
                    if let Some(p_color) = self.pixels[index] {
                        braille_code += 1 << i;
                        cel_color += p_color * (1.0 / 8.0);
                    }
                }
                if braille_code != 0x2800 {
                    string.write_str(&cursor::Goto(
                        (col as u16).saturating_add(1), 
                        (row as u16).saturating_add(1)).to_string()
                    ).unwrap();
                    string.write_str(&Rgb(cel_color.r, cel_color.g, cel_color.b).fg_string()).unwrap();
                    string.write_char(char::from_u32(braille_code).unwrap()).unwrap();
                }
            }
        }
        string
    }
    
    pub fn new() -> Self {
        // let wpos = Command::new("sh")
            // .arg("-c")
            // .arg(r"xdotool getwindowfocus getwindowgeometry --shell | sed /[XYHT]=/P -n | echo -n $(tr -dc '0-9\n')")
            // .output()
            // .expect("failed when getting window position.")
            // .stdout;
        // let wpos: Vec<i32> = from_utf8(&wpos).unwrap().split(" ").map(|d|
            // d.parse::<i32>().unwrap()
        // ).collect();
        // let (win_x, win_y, pix_w, pix_h) = (wpos[0], wpos[1], wpos[2] - 13, wpos[3] - 10);

        let (win_x, win_y, pix_w, pix_h) = (0, 0, 1353, 758);

        let (width, height) = terminal_size().unwrap();
        let (width, height) = (width as usize * 2, height as usize * 4);
        let pixels = vec![None; width * height];
        let projection_matrix = Mat4x4f::projection(
            pix_w as f32 / pix_h as f32, 
            90.0, 
            0.1, 1000.0
        );
        let camera = Camera {
            position: Vec3f::new(0.0, 0.0, 0.0),
            direction: Vec3f::new(0.0, 0.0, 1.0),
        };
        let depth_buffer = vec![f32::MIN; width * height];
        Self { 
            pixels, 
            width, 
            height, 
            win_x, 
            win_y, 
            pix_w, 
            pix_h, 
            projection_matrix, 
            camera,
            depth_buffer,
        }
    }
}

pub struct Camera {
    pub position: Vec3f,
    pub direction: Vec3f,
}