#![no_std]
#![no_main]

use console::console_trait::ConsoleOut;
use core::arch::asm;
use keyboard;
use memory::{MemRegion, dump_memory_map};
use meta::VERSION;

pub struct Shell<C: ConsoleOut + core::fmt::Write> {
    console: C,
    input_buffer: [u8; 128],
    length: usize,
}

impl<C: ConsoleOut + core::fmt::Write> Shell<C> {
    pub fn new(console: C) -> Self {
        Self {
            console: console,
            input_buffer: [0; 128],
            length: 0,
        }
    }

    pub fn run_shell(&mut self) -> ! {
        self.console
            .write_line("Beyond OS v0.0.1 Author: Takahiro Nakamura\n");
        self.console.write_charactor('>');

        loop {
            if let Some(code) = keyboard::pop_scancode() {
                if let Some(char) = keyboard::scancode_to_char(code) {
                    match char {
                        '\n' => {
                            self.console.write_charactor('\n');

                            if self.length != 0 {
                                self.execute_line();
                                // 入力バッファクリア
                                self.length = 0;
                            }
                            // 次のプロンプト出すならここで
                            self.console.write_charactor('>');
                        }
                        '\u{0008}' => {
                            if self.length > 0 {
                                if let Some(_) = self.pop_char() {
                                    self.console.backspace();
                                };
                            }
                        }
                        _ => {
                            self.console.write_charactor(char);
                            self.push_char(char);
                        }
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
                _ => {
                    self.console.write_string("unknown command\n");
                }
            }
        };
    }

    pub fn show_memory_map<I>(&mut self, regions: I)
    where
        I: IntoIterator<Item = MemRegion>,
    {
        dump_memory_map(regions, &mut self.console);
    }
}
