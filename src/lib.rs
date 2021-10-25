pub mod canvas;
pub mod entity;
pub mod shapes;
pub mod vertex;
pub mod math;
pub mod window;

#[cfg(test)]
mod tests {
    use crate::math::Vec3f;

    #[test]
    fn it_works() {
        let window = crate::window::Window::default();
        window.run();
    }
}
