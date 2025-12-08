use crate::color::Color;
use crate::graphics_trait::FrameBuffer;
use bootloader_api::BootInfo;

pub struct BeyondFramebuffer<'a> {
    pub buf: &'a mut [u8],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes_per_pixel: usize,
}

impl<'a> BeyondFramebuffer<'a> {
    pub fn from_boot_info(boot_info: &'a mut BootInfo) -> Option<Self> {
        let fb: &mut bootloader_api::info::FrameBuffer = boot_info.framebuffer.as_mut()?;
        let info: bootloader_api::info::FrameBufferInfo = fb.info();
        let buffer: &mut [u8] = fb.buffer_mut();

        Some(Self {
            buf: buffer,
            width: info.width,
            height: info.height,
            stride: info.stride,
            bytes_per_pixel: info.bytes_per_pixel,
        })
    }
}

impl<'a> FrameBuffer for BeyondFramebuffer<'a> {
    fn put_pixel(&mut self, x: usize, y: usize, c: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx: usize = (y * self.stride + x) * self.bytes_per_pixel;
        self.buf[idx] = c.b;
        self.buf[idx + 1] = c.g;
        self.buf[idx + 2] = c.r;
        if self.bytes_per_pixel == 4 {
            self.buf[idx + 3] = 0x00;
        }
    }

    fn get_pixel(&self, x: usize, y: usize) -> Color {
        if x >= self.width || y >= self.height {
            return Color::black(); // またはデフォルトの色
        }

        // 画面メモリ上の、あるピクセルの“開始位置”を求める計算
        let idx: usize = (y * self.stride + x) * self.bytes_per_pixel;

        let b: u8 = self.buf[idx];
        let g: u8 = self.buf[idx + 1];
        let r: u8 = self.buf[idx + 2];

        Color { r, g, b }
    }

    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn stride(&self) -> usize {
        self.stride
    }
    fn bpp(&self) -> usize {
        self.bytes_per_pixel
    }
}
