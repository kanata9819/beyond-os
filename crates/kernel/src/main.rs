#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use graphics::{color::Color, frame_buffer::BeyondFramebuffer, renderer};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let fb: &mut bootloader_api::info::FrameBuffer =
        boot_info.framebuffer.as_mut().expect("no framebuffer");
    let info: bootloader_api::info::FrameBufferInfo = fb.info();
    let buffer: &mut [u8] = fb.buffer_mut();

    let width: usize = info.width;
    let height: usize = info.height;
    let stride: usize = info.stride;
    let bpp: usize = info.bytes_per_pixel as usize;

    let mut fb: BeyondFramebuffer<'_> = BeyondFramebuffer {
        buf: buffer,
        width: width,
        height: height,
        stride: stride,
        bpp,
    };

    for y in 0..height {
        for x in 0..width {
            fb.put_pixel(x, y, Color::deep_blue());
        }
    }

    let rect_w: usize = width / 3;
    let rect_h: usize = height / 6;
    let start_x: usize = (width - rect_w) / 2;
    let start_y: usize = (height - rect_h) / 2;

    for y in start_y..(start_y + rect_h) {
        for x in start_x..(start_x + rect_w) {
            fb.put_pixel(x, y, Color::white());
        }
    }

    let text: &str = "HELLO BEYOND!";
    let char_w: usize = 8 + 2;
    let text_width: usize = (text.len()) * char_w;
    let text_x: usize = start_x + (rect_w - text_width) / 2;
    let text_y: usize = start_y + rect_h / 2 - 4;

    renderer::draw_text(&mut fb, text_x, text_y, text, Color::black());

    loop {
        unsafe {
            core::arch::asm!("hlt");
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
