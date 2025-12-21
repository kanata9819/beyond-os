#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;
use arch::{idt, interrupts};
use bootloader_api::{
    BootInfo, BootloaderConfig,
    config::Mapping,
    entry_point,
    info::{FrameBuffer, MemoryRegionKind as BlKind, MemoryRegions, Optional},
};
use console::{console::TextConsole, console_trait::Console, serial};
// use core::fmt::Write;
use graphics::{color::Color, frame_buffer::BeyondFramebuffer};
use memory::{MemRegion, MemRegionKind, paging};
use shell::Shell;
use x86_64::{VirtAddr, instructions::interrupts as cpu_int};

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
            serial::init_serial();
            idt::init_idt();
            interrupts::init_interrupts();
            cpu_int::enable();

            let regions_vec: Vec<MemRegion> = convert_regions(regions);
            init_heap(boot_info.physical_memory_offset, &regions_vec);

            Shell::new(
                TextConsole::new(&mut frame_buffer, Color::white(), Color::black()),
                regions_vec,
            )
            .run_shell();
        }
        None => {
            panic!("No FrameBuffer!")
        }
    };
}

fn init_heap(phys_offset: Optional<u64>, regions_vec: &Vec<MemRegion>) {
    if let Some(offset) = phys_offset.into_option() {
        let mut mapper = unsafe { paging::init(VirtAddr::new(offset)) };
        let mut frame_allocator = paging::BootInfoFrameAllocator::new(regions_vec.iter().copied());

        if let Err(e) = memory::init_heap(&mut mapper, &mut frame_allocator) {
            console::serial_println!("heap init failed: {:?}", e);
            panic!("heap init failed");
        }
    }
}

fn convert_regions(regions: &MemoryRegions) -> Vec<MemRegion> {
    let converted = regions.iter().map(|region| MemRegion {
        start: region.start,
        end: region.end,
        kind: match region.kind {
            BlKind::Usable => MemRegionKind::Usable,
            _ => MemRegionKind::Reserved,
        },
    });

    converted.clone().collect()
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
    panic!()
}
