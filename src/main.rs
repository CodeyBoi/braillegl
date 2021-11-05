pub mod canvas;
pub mod entity;
pub mod shapes;
pub mod vertex;
pub mod math;
pub mod window;
pub mod texture;

fn main() {
    let window = crate::window::Window::default();
    window.run();
}