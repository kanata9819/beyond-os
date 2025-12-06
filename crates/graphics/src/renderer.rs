use crate::color::Color;
use crate::font;
use crate::frame_buffer::BeyondFramebuffer;

pub struct Renderer;

impl Renderer {
    pub fn glyph_for(c: char) -> Option<&'static [u8; 8]> {
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

    pub fn render(fb: &mut BeyondFramebuffer) {
        for y in 0..fb.height {
            for x in 0..fb.width {
                fb.put_pixel(x, y, Color::deep_blue());
            }
        }

        let rect_w: usize = fb.width / 3;
        let rect_h: usize = fb.height / 6;
        let start_x: usize = (fb.width - rect_w) / 2;
        let start_y: usize = (fb.height - rect_h) / 2;

        for y in start_y..(start_y + rect_h) {
            for x in start_x..(start_x + rect_w) {
                fb.put_pixel(x, y, Color::white());
            }
        }

        let text: &str = "HELLO BEYOND!";
        let char_w: usize = 8 + 2;
        let text_width: usize = (text.len()) * char_w;
        let text_x: usize = start_x + (rect_w - text_width) / 2;
        let text_y: usize = start_y + rect_h / 2 - 4;

        Self::draw_text(fb, text_x, text_y, text, Color::black());
    }

    pub fn draw_char(
        fb: &mut BeyondFramebuffer,
        x: usize,
        y: usize,
        glyph: &[u8; 8],
        color: Color,
    ) {
        for (row, line) in glyph.iter().enumerate() {
            for col in 0..8 {
                if (line >> (7 - col)) & 1 == 1 {
                    fb.put_pixel(x + col, y + row, color);
                }
            }
        }
    }

    pub fn draw_text(fb: &mut BeyondFramebuffer, x: usize, y: usize, text: &str, color: Color) {
        let mut cx: usize = x;
        for ch in text.chars() {
            if let Some(g) = Self::glyph_for(ch) {
                Self::draw_char(fb, cx, y, g, color);
            }
            cx += 8 + 2;
        }
    }
}
