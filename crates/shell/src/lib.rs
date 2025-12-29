#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use console::console_trait::ConsoleOut;
use core::arch::asm;
use memory::MemRegion;
use meta::VERSION;
use x86_64::{PhysAddr, VirtAddr};

pub mod mem;

pub struct ShellCommands;
impl ShellCommands {
    pub fn enter() -> char {
        '\n'
    }
    pub fn backspace() -> char {
        '\u{0008}'
    }
}

const MAP_TEST_VIRT: u64 = 0x_5555_5555_0000;

pub struct Shell<C: ConsoleOut + core::fmt::Write> {
    regions: Vec<MemRegion>,
    console: C,
    input_buffer: [u8; 128],
    length: usize,
    phys_offset: u64,
}

impl<C: ConsoleOut + core::fmt::Write> Shell<C> {
    pub fn new(console: C, regions: Vec<MemRegion>, phys_offset: u64) -> Self {
        Self {
            regions,
            console,
            input_buffer: [0; 128],
            length: 0,
            phys_offset,
        }
    }

    pub fn run_shell(&mut self) -> ! {
        writeln!(self.console, "Beyond OS v0.1.0 Author: Takahiro Nakamura").unwrap();
        self.console.write_charactor('>');

        loop {
            if let Some(code) = keyboard::pop_scancode()
                && let Some(char) = keyboard::scancode_to_char(code)
            {
                if char == ShellCommands::enter() {
                    self.console.write_charactor('\n');

                    if self.length != 0 {
                        self.execute_line();
                        self.length = 0;
                    }
                    self.console.write_charactor('>');
                } else if char == ShellCommands::backspace() {
                    if self.length > 0 && self.pop_char().is_some() {
                        self.console.backspace();
                    };
                } else {
                    self.console.write_charactor(char);
                    self.push_char(char);
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
                    writeln!(self.console, "welcome to BeyondOS\n").unwrap();
                }
                "help" => {
                    writeln!(self.console, "Show Help\n").unwrap();
                    writeln!(self.console, "hello: to greet to OS").unwrap();
                    writeln!(self.console, "version: to show version of Beyond OS").unwrap();
                    writeln!(self.console, "mem: to show memory map").unwrap();
                    writeln!(
                        self.console,
                        "alloctest(at): to test allocator and show next adderess"
                    )
                    .unwrap();
                    writeln!(
                        self.console,
                        "maptest(mt): map one page and write a test value"
                    )
                    .unwrap();
                }
                "version" => {
                    writeln!(self.console, "{}", VERSION).unwrap();
                }
                "mem" => {
                    mem::show_memory_map(&mut self.console, self.regions.iter().copied());
                }
                "alloctest" | "at" => {
                    let addr = mem::alloc_frame();
                    writeln!(self.console, "{}", addr.unwrap()).unwrap();
                }
                "maptest" | "mt" => {
                    let phys = match mem::alloc_frame() {
                        Some(addr) => addr,
                        None => {
                            writeln!(self.console, "maptest: alloc_frame failed").unwrap();
                            return;
                        }
                    };
                    let mut mapper =
                        unsafe { memory::paging::init(VirtAddr::new(self.phys_offset)) };
                    let mut frame_allocator = memory::paging::GlobalFrameAllocator;
                    match memory::paging::map_one_page(
                        VirtAddr::new(MAP_TEST_VIRT),
                        PhysAddr::new(phys),
                        &mut mapper,
                        &mut frame_allocator,
                    ) {
                        Ok(()) => {
                            unsafe {
                                core::ptr::write_volatile(
                                    MAP_TEST_VIRT as *mut u64,
                                    0x1122_3344_5566_7788,
                                );
                            }
                            writeln!(
                                self.console,
                                "maptest ok virt=0x{:016x} phys=0x{:016x}",
                                MAP_TEST_VIRT, phys
                            )
                            .unwrap();
                        }
                        Err(e) => {
                            writeln!(self.console, "maptest failed: {:?}", e).unwrap();
                        }
                    }
                }
                _ => {
                    writeln!(self.console, "unknown command: {}", line).unwrap();
                }
            }
        };
    }
}
