use graphics::{color::Color, graphics_trait::FrameBuffer};

#[allow(dead_code)]
pub trait Console<'a, FB: FrameBuffer> {
    fn new(fb: &'a mut FB, fg: Color, bg: Color) -> Self;
}
