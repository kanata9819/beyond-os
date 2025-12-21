use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

/// Virtual start address of the heap region.
pub const HEAP_VIRT_START: usize = 0x_4444_4444_0000;
/// Initial heap size mapped at startup.
pub const HEAP_INITIAL_SIZE: usize = 1024 * 1024; // 1 MiB

/// Global allocator instance used by `alloc` types like Box/Vec.
#[global_allocator]
static GLOBAL_ALLOCATOR: BumpAllocator = BumpAllocator;

/// Heap start address (set by `init_heap`).
static HEAP_START: AtomicUsize = AtomicUsize::new(0);
/// Heap end address (set by `init_heap` or `grow_heap`).
static HEAP_END: AtomicUsize = AtomicUsize::new(0);
/// Next allocation pointer (bump cursor).
static NEXT: AtomicUsize = AtomicUsize::new(0);

/// Simple bump allocator with no deallocation.
struct BumpAllocator;

/// Initialize the heap by mapping an initial range of pages.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    map_heap_range(HEAP_VIRT_START, HEAP_INITIAL_SIZE, mapper, frame_allocator)?;

    HEAP_START.store(HEAP_VIRT_START, Ordering::SeqCst);
    HEAP_END.store(HEAP_VIRT_START + HEAP_INITIAL_SIZE, Ordering::SeqCst);
    NEXT.store(HEAP_VIRT_START, Ordering::SeqCst);
    Ok(())
}

/// Grow the heap by mapping additional pages after the current end.
pub fn grow_heap(
    additional_bytes: usize,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    if additional_bytes == 0 {
        return Ok(());
    }

    let old_end = HEAP_END.load(Ordering::SeqCst);
    let size = crate::align_up_usize(additional_bytes, crate::PAGE_SIZE as usize);
    let new_end = old_end
        .checked_add(size)
        .ok_or(MapToError::FrameAllocationFailed)?;

    map_heap_range(old_end, size, mapper, frame_allocator)?;
    HEAP_END.store(new_end, Ordering::SeqCst);
    Ok(())
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start: usize = HEAP_START.load(Ordering::Acquire);

        if start == 0 {
            return null_mut();
        }

        let end: usize = HEAP_END.load(Ordering::Acquire);

        loop {
            let current: usize = NEXT.load(Ordering::Relaxed);
            let alloc_start: usize = crate::align_up_usize(current, layout.align());
            let alloc_end: usize = match alloc_start.checked_add(layout.size()) {
                Some(v) => v,
                None => return null_mut(),
            };

            if alloc_end > end {
                return null_mut();
            }

            if NEXT
                .compare_exchange(current, alloc_end, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return alloc_start as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

fn map_heap_range(
    start: usize,
    size: usize,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let start = crate::align_up_usize(start, crate::PAGE_SIZE as usize);
    let end_unaligned = start
        .checked_add(size)
        .ok_or(MapToError::FrameAllocationFailed)?;
    let end = crate::align_up_usize(end_unaligned, crate::PAGE_SIZE as usize);

    if start >= end {
        return Ok(());
    }

    let start_page = Page::containing_address(VirtAddr::new(start as u64));
    let end_page = Page::containing_address(VirtAddr::new((end - 1) as u64));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    Ok(())
}
