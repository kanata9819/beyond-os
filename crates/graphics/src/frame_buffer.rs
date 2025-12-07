use crate::color::Color;

pub struct BeyondFramebuffer<'a> {
    pub buf: &'a mut [u8],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes_per_pixel: usize,
}

impl<'a> BeyondFramebuffer<'a> {
    pub fn put_pixel(&mut self, x: usize, y: usize, c: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx: usize = ((y * self.stride + x) * self.bytes_per_pixel) as usize;
        self.buf[idx] = c.b;
        self.buf[idx + 1] = c.g;
        self.buf[idx + 2] = c.r;
        if self.bytes_per_pixel == 4 {
            self.buf[idx + 3] = 0x00;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        if x >= self.width || y >= self.height {
            return Color::black(); // またはデフォルトの色
        }

        let idx: usize = (y * self.stride + x) * self.bytes_per_pixel;

        let b: u8 = self.buf[idx];
        let g: u8 = self.buf[idx + 1];
        let r: u8 = self.buf[idx + 2];

        Color { r, g, b }
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn stride(&self) -> usize {
        self.stride
    }
    pub fn bpp(&self) -> usize {
        self.bytes_per_pixel
    }
}
