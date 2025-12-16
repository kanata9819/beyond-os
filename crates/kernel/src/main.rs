#![no_std]
#![no_main]

use arch::{idt, interrupts};
use bootloader_api::{
    BootInfo, entry_point,
    info::{FrameBuffer, MemoryRegionKind as BlKind, MemoryRegions},
};
use console::{console::TextConsole, console_trait::Console};
use graphics::{color::Color, frame_buffer::BeyondFramebuffer};
use memory::{MemRegion, MemRegionKind};
use shell::Shell;
use x86_64::instructions::interrupts as cpu_int;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let regions: &MemoryRegions = &boot_info.memory_regions;
    let frame_buffer: &mut FrameBuffer = boot_info.framebuffer.as_mut().expect("No FrameBudffer!!");

    match BeyondFramebuffer::from_frame_buffer(frame_buffer) {
        Some(mut frame_buffer) => {
            let console = TextConsole::new(&mut frame_buffer, Color::white(), Color::black());
            let mut shell: Shell<TextConsole<'_, BeyondFramebuffer<'_>>> = Shell::new(console);

            idt::init_idt();
            interrupts::init_interrupts();
            cpu_int::enable();

            let converted = regions.iter().map(|r| MemRegion {
                start: r.start,
                end: r.end,
                kind: match r.kind {
                    BlKind::Usable => MemRegionKind::Usable,
                    _ => MemRegionKind::Reserved,
                },
            });

            shell.show_memory_map(converted);
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
    loop {}
}
