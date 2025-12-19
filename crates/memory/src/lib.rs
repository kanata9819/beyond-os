#![no_std]
#![no_main]

use crate::frame::FrameAllocator;

mod frame;
mod heap;

pub const PAGE_SIZE: u64 = 4096;

pub use heap::init_heap;

#[derive(Debug, Clone, Copy)]
pub struct MemRegion {
    pub start: u64,
    pub end: u64,
    pub kind: MemRegionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemRegionKind {
    Usable,
    Reserved,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub start: u64,
    pub end: u64,
}

impl Range {
    pub fn contains(&self, addr: u64) -> bool {
        self.start <= addr && addr < self.end
    }
}

#[inline]
pub fn align_up(x: u64, a: u64) -> u64 {
    debug_assert!(a.is_power_of_two());
    (x + a - 1) & !(a - 1)
}

#[inline]
pub fn align_up_usize(x: usize, a: usize) -> usize {
    debug_assert!(a.is_power_of_two());
    (x + a - 1) & !(a - 1)
}

#[inline]
pub fn align_down(x: u64, a: u64) -> u64 {
    debug_assert!(a.is_power_of_two());
    x & !(a - 1)
}

pub fn alloc_frame<I: IntoIterator<Item = MemRegion>>(
    regions: I,
    console: &mut impl core::fmt::Write,
) {
    let mut allocator = frame::BumpFrameAllocator::new(regions.into_iter());
    if let Some(f) = allocator.alloc_frame() {
        writeln!(console, "alloc {:#x}", f).unwrap();
    }
}

///Output Memory Map
pub fn dump_memory_map<I>(regions: I, console: &mut impl core::fmt::Write)
where
    I: IntoIterator<Item = MemRegion>,
{
    let mut usable_bytes: u64 = 0;

    writeln!(console, "== memory map ==").expect("Cannot Write!");

    for region in regions {
        let start: u64 = region.start;
        let end: u64 = region.end;
        let size: u64 = end - start;

        let kind_str: &str = match region.kind {
            MemRegionKind::Usable => {
                usable_bytes += size;
                "Usable"
            }
            _ => "Reserved/Other",
        };

        writeln!(
            console,
            "0x{:016x} - 0x{:016x} ({} KiB) {}",
            start,
            end,
            size / 1024,
            kind_str
        )
        .ok();
    }

    let usable_mib: u64 = usable_bytes / (1024 * 1024);
    writeln!(console, "Usable total: {} MiB", usable_mib).ok();
}
