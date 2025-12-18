use crate::MemRegion;

pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Option<u64>; // 物理アドレス(4KiB aligned)
}

pub struct BumpFrameAllocator<I>
where
    I: Iterator<Item = MemRegion>,
{
    regions: I,
    current: Option<MemRegion>,
    next_addr: u64,
}

impl<I: Iterator<Item = MemRegion>> BumpFrameAllocator<I> {
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
                    self.next_addr += 0x1000; // 4KiB

                    return Some(addr);
                }
            }

            // 今の region を使い切った or まだ無い → 次の region へ
            let next: MemRegion = self.regions.next()?;
            self.next_addr = next.start;
            self.current = Some(next);
        }
    }
}
