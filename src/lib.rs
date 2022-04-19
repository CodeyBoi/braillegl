pub mod canvas;
pub mod entity;
pub mod shapes;
pub mod vertex;
pub mod math;
pub mod window;
pub mod texture;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let window = crate::window::Window::default();
        window.run();
    }
}
