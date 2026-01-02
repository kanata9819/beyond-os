#![no_std]
#![no_main]

use crate::frame::FrameAllocator;
use spin::Mutex;

mod frame;
mod heap;
pub mod paging;

/// 4 KiB page size used by the memory subsystem.
pub const PAGE_SIZE: u64 = 4096;

/// Initialize the global heap allocator backing store.
pub use heap::{HEAP_INITIAL_SIZE, HEAP_VIRT_START, grow_heap, init_heap};

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

struct MemRegionIter {
    regions: &'static [MemRegion],
    index: usize,
}

impl MemRegionIter {
    fn new(regions: &'static [MemRegion]) -> Self {
        Self { regions, index: 0 }
    }
}

impl Iterator for MemRegionIter {
    type Item = MemRegion;

    fn next(&mut self) -> Option<Self::Item> {
        let region = self.regions.get(self.index)?;
        self.index += 1;
        Some(*region)
    }
}

static FRAME_ALLOCATOR: Mutex<Option<frame::BumpFrameAllocator<MemRegionIter>>> = Mutex::new(None);

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

/// Initialize the global frame allocator from a memory region slice.
pub fn init_frame_allocator(regions: &'static [MemRegion]) {
    let mut allocator = FRAME_ALLOCATOR.lock();
    *allocator = Some(frame::BumpFrameAllocator::new(MemRegionIter::new(regions)));
}

/// Allocate one physical frame from the global allocator.
pub fn alloc_frame() -> Option<u64> {
    let mut allocator = FRAME_ALLOCATOR.lock();
    allocator.as_mut()?.alloc_frame()
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
