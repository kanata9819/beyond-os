use crate::{MemRegion, PAGE_SIZE, align_down, align_up};

/// Physical frame allocator interface.
pub trait FrameAllocator {
    /// Allocate one 4 KiB aligned physical frame.
    ///
    /// Returns the starting physical address on success, or `None` if no
    /// usable frame is available.
    fn alloc_frame(&mut self) -> Option<u64>; // 物理アドレス(4KiB aligned)
}

/// Simple bump allocator over usable memory regions.
pub struct BumpFrameAllocator<I>
where
    I: Iterator<Item = MemRegion>,
{
    regions: I,
    current: Option<MemRegion>,
    next_addr: u64,
}

impl<I: Iterator<Item = MemRegion>> BumpFrameAllocator<I> {
    /// Create a bump allocator from an iterator of memory regions.
    pub fn new(regions: I) -> Self {
        Self {
            regions,
            current: None,
            next_addr: 0,
        }
    }
}

impl<I: Iterator<Item = MemRegion>> FrameAllocator for BumpFrameAllocator<I> {
    fn alloc_frame(&mut self) -> Option<u64> {
        loop {
            if let Some(region) = &self.current {
                // 次に確保するアドレスが、この region の範囲内か？
                if self.next_addr < region.end {
                    let addr: u64 = self.next_addr;
                    self.next_addr += PAGE_SIZE;

                    return Some(addr);
                }
            }

            // 今の region を使い切った or まだ無い → 次の region へ
            let mut next: MemRegion = self.regions.next()?;

            // ページ境界に合わせる
            let start: u64 = align_up(next.start, PAGE_SIZE);
            let end: u64 = align_down(next.end, PAGE_SIZE);
            if start >= end {
                // ページ単位で使えない領域はスキップ
                continue;
            }

            next.start = start;
            next.end = end;

            self.next_addr = next.start;
            self.current = Some(next);
        }
    }
}
