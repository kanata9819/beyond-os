#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use arch::{idt, interrupts};
use bootloader_api::{
    BootInfo, BootloaderConfig,
    config::Mapping,
    entry_point,
    info::{FrameBuffer, MemoryRegionKind as BlKind, MemoryRegions},
};
use console::{console::TextConsole, console_trait::Console, serial};
use core::fmt::Write;
use graphics::{color::Color, frame_buffer::BeyondFramebuffer};
use memory::{MemRegion, MemRegionKind};
use shell::Shell;
use x86_64::{
    VirtAddr,
    instructions::interrupts as cpu_int,
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags},
};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

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

            let phys_offset = boot_info
                .physical_memory_offset
                .into_option()
                .expect("physical memory offset not provided");
            let phys_offset = VirtAddr::new(phys_offset);
            let mut mapper = unsafe { memory::paging::init(phys_offset) };

            let converted = regions.iter().map(|region| MemRegion {
                start: region.start,
                end: region.end,
                kind: match region.kind {
                    BlKind::Usable => MemRegionKind::Usable,
                    _ => MemRegionKind::Reserved,
                },
            });

            let mut frame_allocator =
                memory::paging::BootInfoFrameAllocator::new(converted.clone());

            let page = Page::containing_address(VirtAddr::new(0x_4444_4444_0000));
            let frame = frame_allocator
                .allocate_frame()
                .expect("no usable frame available");
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            let map_result = unsafe { mapper.map_to(page, frame, flags, &mut frame_allocator) };

            match map_result {
                Ok(flush) => {
                    flush.flush();
                    let ptr: *mut u64 = page.start_address().as_mut_ptr();
                    unsafe { ptr.write_volatile(0x_f021_f077_f065_f04e) };
                    console::serial_println!("paging demo: mapped and wrote test value");
                }
                Err(e) => {
                    console::serial_println!("paging demo: map_to failed: {:?}", e);
                }
            }

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

#[cfg(not(test))]
use core::alloc::Layout;
#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    console::serial_println!(
        "alloc_error: size={} align={}",
        layout.size(),
        layout.align()
    );
    loop {}
}
