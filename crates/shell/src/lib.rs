#![no_std]
#![no_main]

pub struct Shell {
    pub input_buffer: [u8; 128],
    pub length: usize,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            input_buffer: [0; 128],
            length: 0,
        }
    }

    pub fn push_char(&mut self, ch: char) {
        if self.length < self.input_buffer.len() {
            self.input_buffer[self.length] = ch as u8; // 今は ASCII 想定でOK
            self.length += 1;
        }
    }

    pub fn pop_char(&mut self) -> Option<char> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            Some(self.input_buffer[self.length] as char)
        }
    }
}
