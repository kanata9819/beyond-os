// use crate::{MemoryMap, RegionKind};
// pub const PAGE_SIZE: u64 = 4096;
//
// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
// pub struct PhysAddr(pub u64);
//
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub struct PhysFrame {
//     pub start: PhysAddr, // 4KiB aligned
// }
//
// #[derive(Clone, Copy, Debug)]
// pub struct Range {
//     pub start: PhysAddr,
//     pub end: PhysAddr, // [start, end)
// }
//
// impl Range {
//     pub fn contains(&self, addr: PhysAddr) -> bool {
//         self.start.0 <= addr.0 && addr.0 < self.end.0
//     }
// }
//
// pub trait FrameAllocator {
//     fn alloc_frame(&mut self) -> Option<PhysFrame>;
// }
//
// pub struct BumpFrameAllocator<'a> {
//     map: MemoryMap<'a>,
//     reserved: &'a [Range],
//     region_index: usize,
//     cur: u64, // current address in region
// }
//
// impl<'a> BumpFrameAllocator<'a> {
//     pub fn new(map: MemoryMap<'a>, reserved: &'a [Range]) -> Self {
//         Self {
//             map,
//             reserved,
//             region_index: 0,
//             cur: 0,
//         }
//     }
//
//     fn is_reserved(&self, addr: u64) -> bool {
//         let a = PhysAddr(addr);
//         self.reserved.iter().any(|r| r.contains(a))
//     }
//
//     fn align_up(x: u64, a: u64) -> u64 {
//         debug_assert!(a.is_power_of_two());
//         (x + a - 1) & !(a - 1)
//     }
// }
//
// impl<'a> FrameAllocator for BumpFrameAllocator<'a> {
//     fn alloc_frame(&mut self) -> Option<PhysFrame> {
//         loop {
//             let region = self.map.regions.get(self.region_index)?;
//
//             if region.kind != RegionKind::Usable {
//                 self.region_index += 1;
//                 self.cur = 0;
//                 continue;
//             }
//
//             if self.cur == 0 {
//                 self.cur = Self::align_up(region.start, PAGE_SIZE);
//             }
//
//             if self.cur + PAGE_SIZE > region.end {
//                 self.region_index += 1;
//                 self.cur = 0;
//                 continue;
//             }
//
//             let addr = self.cur;
//             self.cur += PAGE_SIZE;
//
//             if self.is_reserved(addr) {
//                 continue;
//             }
//
//             return Some(PhysFrame {
//                 start: PhysAddr(addr),
//             });
//         }
//     }
// }
