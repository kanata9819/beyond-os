use crate::console_trait::{Console, ConsoleOut};
use core::fmt::Write;
use graphics::frame_buffer::BeyondFramebuffer;
use graphics::{
    color::Color,
    graphics_trait::FrameBuffer,
    renderer::{self, Renderer},
};
use spin::{Mutex, Once};

pub type KernelConsole = TextConsole<'static, BeyondFramebuffer<'static>>;
/// Handle to the globally-initialized console (locks internally on each write).
pub struct GlobalConsole;

static CONSOLE: Once<Mutex<KernelConsole>> = Once::new();
const GLYPH_W: usize = 8;
const GLYPH_H: usize = 16;
const SCALE: usize = renderer::SCALE;
const CHAR_W: usize = GLYPH_W * SCALE;
const CHAR_H: usize = GLYPH_H * SCALE;
const MARGIN_X: usize = 2;
const MARGIN_Y: usize = 4;

pub struct TextConsole<'a, FB: FrameBuffer> {
    fb: &'a mut FB,
    cursor_col: usize,
    cursor_row: usize,
    cols: usize,
    rows: usize,
    fg: Color,
    bg: Color,
}

pub fn init_console(fb: &'static mut BeyondFramebuffer<'static>) {
    let console: KernelConsole = KernelConsole::new(fb, Color::white(), Color::black());
    CONSOLE.call_once(|| Mutex::new(console));
}

pub fn _print(args: core::fmt::Arguments) {
    if let Some(console) = CONSOLE.get() {
        let mut locked = console.lock();
        locked.write_fmt(args).unwrap();
    }
}

fn with_console<R>(f: impl FnOnce(&mut KernelConsole) -> R) -> Option<R> {
    CONSOLE.get().map(|c| {
        let mut guard = c.lock();
        f(&mut *guard)
    })
}

pub fn global_console() -> Option<GlobalConsole> {
    CONSOLE.get().map(|_| GlobalConsole)
}

impl<'a, FB: FrameBuffer> Write for TextConsole<'a, FB> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.chars() {
            ConsoleOut::write_charactor(self, ch);
        }
        Ok(())
    }
}

impl<'a, FB: FrameBuffer> Console<'a, FB> for TextConsole<'a, FB> {
    fn new(fb: &'a mut FB, fg: Color, bg: Color) -> Self {
        let cols: usize = fb.width() / (CHAR_W + MARGIN_X);
        let rows: usize = fb.height() / (CHAR_H + MARGIN_Y);

        let mut console: TextConsole<'a, FB> = Self {
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
}

impl<'a, FB: FrameBuffer> ConsoleOut for TextConsole<'a, FB> {
    fn write_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_charactor(ch);
        }
    }

    fn write_line(&mut self, s: &str) {
        self.write_string(s);
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

    fn write_charactor(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.newline();
            }
            _ => {
                self.write_charactor_at(ch);
                self.cursor_col += 1;
                if self.cursor_col >= self.cols {
                    self.newline();
                }
            }
        }
    }

    fn write_charactor_at(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.newline();
            }
            _ => {
                if let Some(glyph) = Renderer::glyph_for(ch) {
                    let x: usize = self.cursor_col * (CHAR_W + MARGIN_X);
                    let y: usize = self.cursor_row * (CHAR_H + MARGIN_Y) + MARGIN_Y;
                    Renderer::draw_char(self.fb, x, y, glyph, self.fg);
                }
            }
        }
    }

    fn newline(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;

        if self.cursor_row >= self.rows {
            self.scroll_up();
            self.cursor_row = self.rows - 1;
        }
    }

    fn scroll_up(&mut self) {
        let row_h: usize = CHAR_H + MARGIN_Y;

        for y in 0..(self.fb.height() - row_h) {
            for x in 0..self.fb.width() {
                let color: Color = self.fb.get_pixel(x, y + row_h);
                self.fb.put_pixel(x, y, color);
            }
        }

        for y in (self.fb.height() - row_h)..self.fb.height() {
            for x in 0..self.fb.width() {
                self.fb.put_pixel(x, y, self.bg);
            }
        }
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cols - 1;
        }

        Self::erase_cell(self);
    }

    fn erase_cell(&mut self) {
        let x0: usize = self.cursor_col * (CHAR_W + MARGIN_X);
        let y0: usize = self.cursor_row * (CHAR_H + MARGIN_Y) + MARGIN_Y;
        for y in 0..CHAR_H {
            for x in 0..CHAR_W {
                self.fb.put_pixel(x0 + x, y0 + y, self.bg);
            }
        }
    }
}

impl Write for GlobalConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let _ = with_console(|c| c.write_str(s)).ok_or(core::fmt::Error); // propagate missing console as error
        Ok(())
    }
}

impl ConsoleOut for GlobalConsole {
    fn write_string(&mut self, s: &str) {
        let _ = with_console(|c| c.write_string(s));
    }

    fn write_line(&mut self, s: &str) {
        let _ = with_console(|c| c.write_line(s));
    }

    fn clear(&mut self) {
        let _ = with_console(|c| c.clear());
    }

    fn write_charactor(&mut self, ch: char) {
        let _ = with_console(|c| c.write_charactor(ch));
    }

    fn write_charactor_at(&mut self, ch: char) {
        let _ = with_console(|c| c.write_charactor_at(ch));
    }

    fn backspace(&mut self) {
        let _ = with_console(|c| c.backspace());
    }

    fn newline(&mut self) {
        let _ = with_console(|c| c.newline());
    }

    fn scroll_up(&mut self) {
        let _ = with_console(|c| c.scroll_up());
    }

    fn erase_cell(&mut self) {
        let _ = with_console(|c| c.erase_cell());
    }
}
