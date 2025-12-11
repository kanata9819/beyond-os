#![no_std]
#![no_main]

use console::console_trait::ConsoleOut;
use keyboard;

pub struct Shell<C: ConsoleOut> {
    console: C,
    input_buffer: [u8; 128],
    length: usize,
}

impl<C: ConsoleOut> Shell<C> {
    pub fn new(console: C) -> Self {
        Self {
            console: console,
            input_buffer: [0; 128],
            length: 0,
        }
    }

    pub fn run_shell(&mut self) -> ! {
        self.console
            .write_string("Beyond OS v0.0.1 Author: Takahiro Nakamura\n");

        loop {
            if let Some(code) = keyboard::pop_scancode() {
                if let Some(char) = keyboard::scancode_to_char(code) {
                    self.console.write_charactor(char);
                }
            }

            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }

    pub fn push_char(&mut self, ch: char) {
        if self.length < self.input_buffer.len() {
            self.input_buffer[self.length] = ch as u8;
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
