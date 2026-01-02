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
mod virtio_blk;
use x86_64::{VirtAddr, instructions::interrupts as cpu_int};

// IDs and sizes from PCI/Virtio specs.
const VIRTIO_VENDOR_ID: u16 = 0x1af4;
const VIRTIO_BLK_DEVICE_ID_LEGACY: u16 = 0x1001;
const PCI_BAR_COUNT: u8 = 6;
const VIRTIO_LEGACY_BAR_INDEX: u8 = 0;
const SECTOR_SIZE_BYTES: usize = 512;
const BOOT_SECTOR_LBA: u64 = 0;

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
    let phys_offset = boot_info.physical_memory_offset.into_option();
    let regions_for_allocator = convert_regions(regions);

    match BeyondFramebuffer::from_frame_buffer(frame_buffer) {
        Some(mut frame_buffer) => {
            serial::init_serial();
            idt::init_idt();
            interrupts::init_interrupts();
            cpu_int::enable();
            init_heap(phys_offset, regions);
            memory::init_frame_allocator(regions_for_allocator.clone().leak());

            serial_println!("PCI scan:");
            pci::scan(|dev| {
                if dev.vendor_id == VIRTIO_VENDOR_ID {
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
                    for bar_index in 0u8..PCI_BAR_COUNT {
                        if let Some(bar) =
                            pci::read_bar(dev.bus, dev.device, dev.function, bar_index)
                        {
                            serial_println!(
                                "  bar{}: kind={:?} base=0x{:x} size={:?}",
                                bar_index,
                                bar.kind,
                                bar.base,
                                bar.size
                            );
                        }
                    }
                    if dev.device_id == VIRTIO_BLK_DEVICE_ID_LEGACY
                        && let Some(bar) = pci::read_bar(
                            dev.bus,
                            dev.device,
                            dev.function,
                            VIRTIO_LEGACY_BAR_INDEX,
                        )
                        && bar.kind == arch::pci::BarKind::Io
                    {
                        pci::enable_io_bus_master(dev.bus, dev.device, dev.function);
                        if let Some(offset) = phys_offset {
                            match virtio_blk::init_legacy(bar.base as u16, offset) {
                                Ok(mut blk) => {
                                    serial_println!(
                                        "virtio-blk capacity: {} sectors",
                                        blk.capacity_sectors()
                                    );
                                    let mut buf = [0u8; SECTOR_SIZE_BYTES];
                                    if blk.read_sector(BOOT_SECTOR_LBA, &mut buf).is_ok() {
                                        serial_println!("virtio-blk read sector 0 ok");
                                    } else {
                                        serial_println!("virtio-blk read sector 0 failed");
                                    }
                                }
                                Err(e) => {
                                    serial_println!("virtio-blk init failed: {}", e);
                                }
                            }
                        } else {
                            serial_println!("virtio-blk: no physical memory offset");
                        }
                    }
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

            Shell::new(
                TextConsole::new(&mut frame_buffer, Color::white(), Color::black()),
                regions_for_allocator,
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
