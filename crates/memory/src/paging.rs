use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
        mapper::MapToError,
    },
};

use crate::{MemRegion, MemRegionKind, PAGE_SIZE, align_down, align_up};

/// Initialize an `OffsetPageTable` using the current active level 4 page table.
///
/// # Safety
/// The `physical_memory_offset` must map the complete physical memory, as provided
/// by the bootloader config. Using the wrong offset will cause undefined behavior.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table: &'static mut PageTable =
        unsafe { active_level_4_table(physical_memory_offset) };
    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}

/// Frame allocator backed by the global `alloc_frame` bump allocator.
pub struct GlobalFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let addr = crate::alloc_frame()?;
        Some(PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

/// Map a single 4 KiB page to the specified physical frame.
pub fn map_one_page(
    virt: VirtAddr,
    phys: PhysAddr,
    mapper: &mut OffsetPageTable<'static>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    use x86_64::structures::paging::Mapper;

    let page = Page::containing_address(virt);
    let frame = PhysFrame::containing_address(phys);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    Ok(())
}

/// Returns a mutable reference to the active level 4 page table.
///
/// # Safety
/// The caller must ensure that the returned reference is unique and the offset is correct.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}

/// Frame allocator that hands out usable 4 KiB frames from the boot memory map.
pub struct BootInfoFrameAllocator<I>
where
    I: Iterator<Item = MemRegion>,
{
    regions: I,
    current: Option<MemRegion>,
    next_addr: u64,
}

impl<I: Iterator<Item = MemRegion>> BootInfoFrameAllocator<I> {
    /// Create a new frame allocator from a memory region iterator.
    pub fn new(regions: I) -> Self {
        Self {
            regions,
            current: None,
            next_addr: 0,
        }
    }

    fn next_usable_region(&mut self) -> Option<MemRegion> {
        for mut region in self.regions.by_ref() {
            if region.kind != MemRegionKind::Usable {
                continue;
            }

            let start = align_up(region.start, PAGE_SIZE);
            let end = align_down(region.end, PAGE_SIZE);
            if start >= end {
                continue;
            }

            region.start = start;
            region.end = end;
            return Some(region);
        }
        None
    }
}

unsafe impl<I: Iterator<Item = MemRegion>> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<I> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        loop {
            if let Some(region) = &self.current
                && self.next_addr < region.end
            {
                let addr = self.next_addr;
                self.next_addr += PAGE_SIZE;
                let phys = PhysAddr::new(addr);
                return Some(PhysFrame::containing_address(phys));
            }

            let next = self.next_usable_region()?;
            self.next_addr = next.start;
            self.current = Some(next);
        }
    }
}
