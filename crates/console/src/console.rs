use graphics::{color::Color, frame_buffer::BeyondFramebuffer, renderer::Renderer};

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
        const PIXEL: usize = 8;
        let cols: usize = fb.width() / PIXEL;
        let rows: usize = fb.height() / PIXEL;

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

    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    pub fn write_line(&mut self, s: &str) {
        self.write_str(s);
        self.write_char('\n');
    }

    fn clear(&mut self) {
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

    pub fn write_char(&mut self, ch: char) {
        const MARGIN_X: usize = 16;
        const MARGIN_Y: usize = 16;
        match ch {
            '\n' => {
                self.newline();
                self.write_char('>');
            }
            _ => {
                if let Some(glyph) = Renderer::glyph_for(ch) {
                    let x: usize = self.cursor_col * (8 + MARGIN_X);
                    let y: usize = self.cursor_row * 24 + MARGIN_Y;
                    Renderer::draw_char(self.fb, x, y, glyph, self.fg);
                }

                self.cursor_col += 1;
                if self.cursor_col >= self.cols {
                    self.newline();
                }
            }
        }
    }

    pub fn newline(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;

        if self.cursor_row >= self.rows {
            self.scroll_up();
            self.cursor_row = self.rows - 1;
        }
    }

    fn scroll_up(&mut self) {
        let row_h: usize = 8;

        for y in 0..(self.fb.height - row_h) {
            for x in 0..self.fb.width {
                let color: Color = self.fb.get_pixel(x, y + row_h);
                self.fb.put_pixel(x, y, color);
            }
        }

        for y in (self.fb.height - row_h)..self.fb.height {
            for x in 0..self.fb.width {
                self.fb.put_pixel(x, y, self.bg);
            }
        }
    }
}
