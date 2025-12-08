use crate::color::Color;

pub trait FrameBuffer {
    fn put_pixel(&mut self, x: usize, y: usize, c: Color);
    fn get_pixel(&self, x: usize, y: usize) -> Color;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn stride(&self) -> usize;
    fn bpp(&self) -> usize;
}
