use crate::color::Color;
use crate::font;
use crate::frame_buffer::BeyondFramebuffer;

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

pub fn draw_char(fb: &mut BeyondFramebuffer, x: usize, y: usize, glyph: &[u8; 8], color: Color) {
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
        if let Some(g) = glyph_for(ch) {
            draw_char(fb, cx, y, g, color);
        }
        cx += 8 + 2; // 文字幅 + すきま
    }
}
