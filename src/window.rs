use std::{io::{self, Write}, time::{Duration, Instant}};

use device_query::{DeviceQuery, DeviceState};
use termion::{clear, color::{Color, White}, cursor, input::MouseTerminal, raw::IntoRawMode};

use crate::{canvas::Canvas, entity::Entity, math::Vec3f, shapes};

pub struct Window { }

impl Window {

    pub fn default() -> Self {
        Self {  }
    }

    pub fn run(&self) {
        let mut canvas = Canvas::new();
        
        let d_state = DeviceState::new();
        let mut prev_mouse = d_state.get_mouse();
        
        // Load geometry
        let mut ball = Entity::with_geometry(
            // shapes::make_sphere(4.0, 25, 25)
            shapes::load_from_file("res/squidmask.obj")
        );
        ball.set_translation(Vec3f::new(0.0, 0.0, -30.0));
        let mut ball_rot = 0.0;

        // Define user constants
        let preferred_fps = 60;

        let millis_between_ticks = 1000 / (preferred_fps + 2);
        let mut tick: u64 = 0;
        let time = Instant::now();

        // Set terminal to raw mode
        let mut _stdout = MouseTerminal::from(
            io::stdout().into_raw_mode().unwrap()
        );
        print!("{}{}", cursor::Hide, clear::All);

        'main: loop {
            // Update time
            let t = time.elapsed().as_secs_f32();

            // Get input state
            let mouse = d_state.get_mouse();
            let keys = d_state.get_keys();
            
            // Handle events
            for k in &keys {
                use device_query::Keycode::*;
                match k {
                    Escape => break 'main,
                    W => ball.translate(Vec3f::new(0.0, 0.15, 0.0)),
                    S => ball.translate(Vec3f::new(0.0, -0.15, 0.0)),
                    A => ball.translate(Vec3f::new(0.15, 0.0, 0.0)),
                    D => ball.translate(Vec3f::new(-0.15, 0.0, 0.0)),
                    Q => ball.translate(Vec3f::new(0.0, 0.0, -0.15)),
                    E => ball.translate(Vec3f::new(0.0, 0.0, 0.15)),
                    _ => {},
                }
            }
            // if mouse.button_pressed[1] && prev_mouse.button_pressed[1] {
            //     let (mx, my) = canvas.pix2cell(mouse.coords);
            //     let (pmx, pmy) = canvas.pix2cell(prev_mouse.coords);
            //     canvas.draw_line(pmx, pmy, mx, my);
            // }

            // Update positions
            ball_rot += 0.01;
            ball.set_rotation_y(ball_rot);

            // Render
            canvas.clear();
            canvas.draw_entity(&ball);
            print!("{}{}{}fps={}", canvas.to_s(), cursor::Goto(1, 1), White.fg_str(), (tick as f32 / t) as u64);
            io::stdout().flush().unwrap();

            // Save states for next frame
            prev_mouse = mouse;
            tick += 1;
            let frame_time = ((time.elapsed().as_secs_f32() - t) * 1000.0) as u64;
            let frame_time = if frame_time >= millis_between_ticks {
                millis_between_ticks
            } else {
                frame_time
            };
            std::thread::sleep(Duration::from_millis(millis_between_ticks - frame_time));
        }
        print!("{}{}{}{}", White.fg_str(), clear::All, cursor::Goto(1, 1), cursor::Show);
        io::stdout().flush().unwrap();

        drop(_stdout);
    }
}