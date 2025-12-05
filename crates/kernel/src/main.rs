#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use color::Color;
mod color;
mod font;

entry_point!(kernel_main);

struct SimpleFramebuffer<'a> {
    buf: &'a mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    bpp: usize,
}

impl<'a> SimpleFramebuffer<'a> {
    fn put_pixel(&mut self, x: usize, y: usize, c: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.stride + x) * self.bpp) as usize;
        self.buf[idx] = c.b;
        self.buf[idx + 1] = c.g;
        self.buf[idx + 2] = c.r;
        if self.bpp == 4 {
            self.buf[idx + 3] = 0x00;
        }
    }
}

fn glyph_for(c: char) -> Option<&'static [u8; 8]> {
    match c {
        'H' => Some(&font::GLYPH_H),
        'E' => Some(&font::GLYPH_E),
        'L' => Some(&font::GLYPH_L),
        'O' => Some(&font::GLYPH_O),
        'B' => Some(&font::GLYPH_B),
        'Y' => Some(&font::GLYPH_Y),
        'N' => Some(&font::GLYPH_N),
        'D' => Some(&font::GLYPH_D),
        '!' => Some(&font::GLYPH_EXCL),
        ' ' => None,
        _ => None,
    }
}

fn draw_char(fb: &mut SimpleFramebuffer, x: usize, y: usize, glyph: &[u8; 8], color: Color) {
    for (row, line) in glyph.iter().enumerate() {
        for col in 0..8 {
            if (line >> (7 - col)) & 1 == 1 {
                fb.put_pixel(x + col, y + row, color);
            }
        }
    }
}

fn draw_text(fb: &mut SimpleFramebuffer, x: usize, y: usize, text: &str, color: Color) {
    let mut cx = x;
    for ch in text.chars() {
        if let Some(g) = glyph_for(ch) {
            draw_char(fb, cx, y, g, color);
        }
        cx += 8 + 2; // 文字幅 + すきま
    }
}

/* ============== ここからエントリポイント ============== */
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let fb = boot_info.framebuffer.as_mut().expect("no framebuffer");
    let info = fb.info();
    let buffer = fb.buffer_mut();

    let width = info.width;
    let height = info.height;
    let stride = info.stride;
    let bpp = info.bytes_per_pixel as usize;

    let mut fb = SimpleFramebuffer {
        buf: buffer,
        width: width,
        height: height,
        stride: stride,
        bpp,
    };

    // 背景：濃い青
    let bg = Color {
        r: 0x10,
        g: 0x40,
        b: 0x60,
    };
    for y in 0..height {
        for x in 0..width {
            fb.put_pixel(x, y, bg);
        }
    }

    // 中央の黄色い箱
    let rect_w = width / 3;
    let rect_h = height / 6;
    let start_x = (width - rect_w) / 2;
    let start_y = (height - rect_h) / 2;

    for y in start_y..(start_y + rect_h) {
        for x in start_x..(start_x + rect_w) {
            fb.put_pixel(x, y, Color::white());
        }
    }

    // 文字列 "HELLO BEYOND!"
    let text: &str = "HELLO BEYOND!";
    let char_w: usize = 8 + 2; // 文字 + 余白
    let text_width: usize = (text.len()) * char_w;
    let text_x: usize = start_x + (rect_w - text_width) / 2;
    let text_y: usize = start_y + rect_h / 2 - 4; // だいたい中央

    draw_text(&mut fb, text_x, text_y, text, color::Color::black());

    // そのまま停止
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
