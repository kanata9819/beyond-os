#![allow(dead_code)]

use crate::{color::Color, frame_buffer::BeyondFramebuffer};

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
