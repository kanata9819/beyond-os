#![no_std]
#![no_main]

mod vga_buffer;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    vga_buffer::print("Hello Beyond!");

    loop {
        // OSなので、終わらずにループし続ける
    }
}

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
