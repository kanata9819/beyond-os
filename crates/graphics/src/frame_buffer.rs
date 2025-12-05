use crate::color::Color;
pub struct BeyondFramebuffer<'a> {
    pub buf: &'a mut [u8],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bpp: usize,
}

impl<'a> BeyondFramebuffer<'a> {
    pub fn put_pixel(&mut self, x: usize, y: usize, c: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx: usize = ((y * self.stride + x) * self.bpp) as usize;
        self.buf[idx] = c.b;
        self.buf[idx + 1] = c.g;
        self.buf[idx + 2] = c.r;
        if self.bpp == 4 {
            self.buf[idx + 3] = 0x00;
        }
    }
}
