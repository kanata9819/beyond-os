#![allow(dead_code)]

use crate::{color::Color, frame_buffer::BeyondFramebuffer, renderer::Renderer};

pub struct Console<'a> {
    fb: &'a mut BeyondFramebuffer<'a>,
    cursor_col: usize,
    cursor_row: usize,
    cols: usize,
    rows: usize,
    fg: Color,
    bg: Color,
}

impl<'a> Console<'a> {
    pub fn new(fb: &'a mut BeyondFramebuffer<'a>, fg: Color, bg: Color) -> Self {
        let cols: usize = fb.width() as usize / 8;
        let rows: usize = fb.height() as usize / 8;

        let mut console: Console<'a> = Self {
            fb,
            cursor_col: 0,
            cursor_row: 0,
            cols,
            rows,
            fg,
            bg,
        };

        console.clear();
        console
    }

    fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.newline();
            }
            _ => {
                if let Some(glyph) = Renderer::glyph_for(ch) {
                    let x: usize = self.cursor_col * (8 + 2); // 8px + 2px余白
                    let y: usize = self.cursor_row * 8;
                    Renderer::draw_char(self.fb, x, y, glyph, self.fg);
                }

                self.cursor_col += 1;
                if self.cursor_col >= self.cols {
                    self.newline();
                }
            }
        }
    }

    fn newline(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row + 1 >= self.rows {
            // v1：本物のスクロールは後回し。いったん全クリアでOK
            self.clear();
        } else {
            self.cursor_row += 1;
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    pub fn write_line(&mut self, s: &str) {
        self.write_str(s);
        self.write_char('\n');
    }

    pub fn clear(&mut self) {
        let w: usize = self.fb.width();
        let h: usize = self.fb.height();
        for y in 0..h {
            for x in 0..w {
                self.fb.put_pixel(x, y, self.bg);
            }
        }
        self.cursor_col = 0;
        self.cursor_row = 0;
    }
}
