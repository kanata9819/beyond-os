#![no_std]
#![no_main]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use arch::{idt, interrupts};
use bootloader_api::{
    BootInfo, entry_point,
    info::{FrameBuffer, MemoryRegionKind as BlKind, MemoryRegions},
};
use console::{console::TextConsole, console_trait::Console, serial};
use core::fmt::Write;
use graphics::{color::Color, frame_buffer::BeyondFramebuffer};
use memory::{MemRegion, MemRegionKind};
use shell::Shell;
use x86_64::instructions::interrupts as cpu_int;

entry_point!(kernel_main);

/// entry_point of BeyondOS
/// recieve Memory Regions and FrameBuffer from bootloader_api.
/// init idt(Interrupt Descriptor Table) and then interrupter of x86_64 crates enable.
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let regions: &MemoryRegions = &boot_info.memory_regions;
    let frame_buffer: &mut FrameBuffer = boot_info.framebuffer.as_mut().expect("No FrameBudffer!!");

    match BeyondFramebuffer::from_frame_buffer(frame_buffer) {
        Some(mut frame_buffer) => {
            let mut console: TextConsole<'_, BeyondFramebuffer<'_>> =
                TextConsole::new(&mut frame_buffer, Color::white(), Color::black());

            serial::init_serial();
            console::serial_println!("serial online");
            memory::init_heap();

            let boxed: Box<u64> = Box::new(1234);
            let mut v: Vec<u64> = Vec::new();
            v.push(10);
            v.push(20);
            writeln!(console, "heap demo: boxed={}, vec={:?}", *boxed, v).ok();

            let mut shell: Shell<TextConsole<'_, BeyondFramebuffer<'_>>> = Shell::new(console);

            idt::init_idt();
            interrupts::init_interrupts();
            cpu_int::enable();

            let converted = regions.iter().map(|region| MemRegion {
                start: region.start,
                end: region.end,
                kind: match region.kind {
                    BlKind::Usable => MemRegionKind::Usable,
                    _ => MemRegionKind::Reserved,
                },
            });

            shell.show_memory_map(converted.clone());
            shell.alloc(converted);
            shell.run_shell();
        }
        None => {
            panic!("No FrameBuffer!")
        }
    };
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    console::serial_println!("panic: {}", _info);
    loop {}
}
