#![no_std]
#![no_main]

use crate::frame::FrameAllocator;

mod frame;
mod heap;

/// 4 KiB page size used by the memory subsystem.
pub const PAGE_SIZE: u64 = 4096;

/// Initialize the global heap allocator backing store.
pub use heap::init_heap;

/// Memory region description provided by the bootloader.
#[derive(Debug, Clone, Copy)]
pub struct MemRegion {
    pub start: u64,
    pub end: u64,
    pub kind: MemRegionKind,
}

/// Memory region classification used by the kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemRegionKind {
    Usable,
    Reserved,
    Other,
}

/// Generic address range [start, end) used for range checks.
#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub start: u64,
    pub end: u64,
}

impl Range {
    /// Returns true if `addr` is inside this range.
    pub fn contains(&self, addr: u64) -> bool {
        self.start <= addr && addr < self.end
    }
}

#[inline]
/// Align `x` up to the next multiple of `a` (power of two).
pub fn align_up(x: u64, a: u64) -> u64 {
    debug_assert!(a.is_power_of_two());
    (x + a - 1) & !(a - 1)
}

#[inline]
/// Align `x` up to the next multiple of `a` (power of two), usize variant.
pub fn align_up_usize(x: usize, a: usize) -> usize {
    debug_assert!(a.is_power_of_two());
    (x + a - 1) & !(a - 1)
}

#[inline]
/// Align `x` down to the previous multiple of `a` (power of two).
pub fn align_down(x: u64, a: u64) -> u64 {
    debug_assert!(a.is_power_of_two());
    x & !(a - 1)
}

#[allow(unused_variables)]
/// Allocate one physical frame from the provided memory regions.
pub fn alloc_frame<I: IntoIterator<Item = MemRegion>>(
    regions: I,
    console: &mut impl core::fmt::Write,
) {
    let mut allocator = frame::BumpFrameAllocator::new(regions.into_iter());
    allocator.alloc_frame();
}

/// Dump the bootloader memory map to the provided console.
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
