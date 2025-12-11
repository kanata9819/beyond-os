#![no_std]
#![no_main]

use arch::{idt, interrupts};
use bootloader_api::{entry_point, BootInfo};
use console::{console::TextConsole, console_trait::Console};
use graphics::{color::Color, frame_buffer::BeyondFramebuffer};
use shell::Shell;
use x86_64::instructions::interrupts as cpu_int;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    match BeyondFramebuffer::from_boot_info(boot_info) {
        Some(mut frame_buffer) => {
            let console: TextConsole<'_, BeyondFramebuffer<'_>> =
                TextConsole::new(&mut frame_buffer, Color::white(), Color::black());
            let mut shell: Shell<TextConsole<'_, BeyondFramebuffer<'_>>> = Shell::new(console);

            idt::init_idt();
            interrupts::init_interrupts();
            cpu_int::enable();

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
