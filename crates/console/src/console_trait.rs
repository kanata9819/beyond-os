use graphics::{color::Color, graphics_trait::FrameBuffer};

#[allow(dead_code)]
pub trait Console<'a, FB: FrameBuffer> {
    fn new(fb: &'a mut FB, fg: Color, bg: Color) -> Self;
}

pub trait ConsoleOut {
    fn write_str(&mut self, s: &str);
    fn write_line(&mut self, s: &str);
    fn clear(&mut self);
    fn write_char(&mut self, ch: char);
    fn newline(&mut self);
    fn scroll_up(&mut self);
}
