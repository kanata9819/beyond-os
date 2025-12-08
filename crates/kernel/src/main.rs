#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use console::console::TextConsole;
use graphics::{color::Color, frame_buffer::BeyondFramebuffer};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let fb: &mut bootloader_api::info::FrameBuffer =
        boot_info.framebuffer.as_mut().expect("no framebuffer");
    let info: bootloader_api::info::FrameBufferInfo = fb.info();
    let buffer: &mut [u8] = fb.buffer_mut();

    let mut fb: BeyondFramebuffer<'_> = BeyondFramebuffer {
        buf: buffer,
        width: info.width,
        height: info.height,
        stride: info.stride,
        bytes_per_pixel: info.bytes_per_pixel,
    };

    let mut console: TextConsole<'_, BeyondFramebuffer<'_>> =
        TextConsole::new(&mut fb, Color::white(), Color::black());

    console.write_str("Beyond OS v0.0.1 Author: Takahiro Nakamura\n");

    loop {
        unsafe {
            if let Some(code) = keyboard::read_scancode() {
                if let Some(char) = keyboard::scancode_to_char(code) {
                    console.write_char(char);
                }
            }
        }
    }
}

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
