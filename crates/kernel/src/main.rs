#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use console::{console::TextConsole, console_trait::Console};
use graphics::{color::Color, frame_buffer::BeyondFramebuffer};
use shell::Shell;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    match BeyondFramebuffer::from_boot_info(boot_info) {
        Some(mut frame_buffer) => {
            let console: TextConsole<'_, BeyondFramebuffer<'_>> =
                TextConsole::new(&mut frame_buffer, Color::white(), Color::black());
            let mut shell: Shell<TextConsole<'_, BeyondFramebuffer<'_>>> = Shell::new(console);

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
