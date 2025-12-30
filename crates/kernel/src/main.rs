#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;
use arch::{idt, interrupts, pci};
use bootloader_api::{
    BootInfo, BootloaderConfig,
    config::Mapping,
    entry_point,
    info::{FrameBuffer, MemoryRegionKind as BlKind, MemoryRegions},
};
use console::{console::TextConsole, console_trait::Console, serial, serial_println};
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
            let phys_offset = boot_info.physical_memory_offset.into_option();
            init_heap(phys_offset, regions);

            serial_println!("PCI scan:");
            pci::scan(|dev| {
                if dev.vendor_id == 0x1af4 {
                    serial_println!(
                        "pci {:02x}:{:02x}.{:x} vendor={:04x} device={:04x} class={:02x} subclass={:02x} prog_if={:02x} (virtio)",
                        dev.bus,
                        dev.device,
                        dev.function,
                        dev.vendor_id,
                        dev.device_id,
                        dev.class_code,
                        dev.subclass,
                        dev.prog_if
                    );
                } else {
                    serial_println!(
                        "pci {:02x}:{:02x}.{:x} vendor={:04x} device={:04x} class={:02x} subclass={:02x} prog_if={:02x}",
                        dev.bus,
                        dev.device,
                        dev.function,
                        dev.vendor_id,
                        dev.device_id,
                        dev.class_code,
                        dev.subclass,
                        dev.prog_if
                    );
                }
            });

            let regions_for_allocator = convert_regions(regions);
            let regions_for_shell = regions_for_allocator.clone();
            let regions_slice: &'static [MemRegion] = regions_for_allocator.leak();
            memory::init_frame_allocator(regions_slice);

            Shell::new(
                TextConsole::new(&mut frame_buffer, Color::white(), Color::black()),
                regions_for_shell,
                phys_offset.expect("No physical memory offset"),
            )
            .run_shell();
        }
        None => {
            panic!("No FrameBuffer!")
        }
    };
}

fn init_heap(phys_offset: Option<u64>, regions: &MemoryRegions) {
    if let Some(offset) = phys_offset {
        let mut mapper = unsafe { paging::init(VirtAddr::new(offset)) };

        let iter = regions.iter().map(|region| MemRegion {
            start: region.start,
            end: region.end,
            kind: match region.kind {
                BlKind::Usable => MemRegionKind::Usable,
                _ => MemRegionKind::Reserved,
            },
        });

        let mut frame_allocator = paging::BootInfoFrameAllocator::new(iter);

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

    converted.collect()
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
