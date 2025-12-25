#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use console::console_trait::ConsoleOut;
use core::arch::asm;
use memory::MemRegion;
use meta::VERSION;

pub mod mem;

pub struct Shell<C: ConsoleOut + core::fmt::Write> {
    regions: Vec<MemRegion>,
    console: C,
    input_buffer: [u8; 128],
    length: usize,
}

impl<C: ConsoleOut + core::fmt::Write> Shell<C> {
    pub fn new(console: C, regions: Vec<MemRegion>) -> Self {
        Self {
            regions,
            console,
            input_buffer: [0; 128],
            length: 0,
        }
    }

    pub fn run_shell(&mut self) -> ! {
        self.console
            .write_line("Beyond OS v0.1.0 Author: Takahiro Nakamura\n");
        self.console.write_charactor('>');

        loop {
            if let Some(code) = keyboard::pop_scancode()
                && let Some(char) = keyboard::scancode_to_char(code)
            {
                match char {
                    '\n' => {
                        self.console.write_charactor('\n');

                        if self.length != 0 {
                            self.execute_line();
                            self.length = 0;
                        }
                        self.console.write_charactor('>');
                    }
                    '\u{0008}' => {
                        if self.length > 0 && self.pop_char().is_some() {
                            self.console.backspace();
                        };
                    }
                    _ => {
                        self.console.write_charactor(char);
                        self.push_char(char);
                    }
                }
            }

            unsafe {
                asm!("hlt");
            }
        }
    }

    fn push_char(&mut self, ch: char) {
        if self.length < self.input_buffer.len() {
            self.input_buffer[self.length] = ch as u8;
            self.length += 1;
        }
    }

    fn pop_char(&mut self) -> Option<char> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            Some(self.input_buffer[self.length] as char)
        }
    }

    fn execute_line(&mut self) {
        let bytes: &[u8] = &self.input_buffer[..self.length];
        if let Ok(line) = str::from_utf8(bytes) {
            match line {
                "hello" => {
                    self.console.write_string("welcome to BeyondOS\n");
                }
                "help" => {
                    self.console.write_string("Show Help\n");
                    self.console.write_string("put hello to greet to OS\n");
                }
                "version" => {
                    self.console.write_string(VERSION);
                    self.console.write_charactor('\n');
                }
                "mem" => {
                    mem::show_memory_map(&mut self.console, self.regions.iter().copied());
                }
                "alloc" => {
                    mem::alloc_frame(&mut self.console, self.regions.iter().copied());
                }
                _ => {
                    self.console.write_string("unknown command\n");
                }
            }
        };
    }
}
