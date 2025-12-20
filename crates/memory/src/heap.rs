use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Fixed heap size used by the bump allocator.
const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

/// Global allocator instance used by `alloc` types like Box/Vec.
#[global_allocator]
static GLOBAL_ALLOCATOR: BumpAllocator = BumpAllocator;

/// Heap start address (set by `init_heap`).
static HEAP_START: AtomicUsize = AtomicUsize::new(0);
/// Heap end address (set by `init_heap`).
static HEAP_END: AtomicUsize = AtomicUsize::new(0);
/// Next allocation pointer (bump cursor).
static NEXT: AtomicUsize = AtomicUsize::new(0);

/// Simple bump allocator with no deallocation.
struct BumpAllocator;

/// Static heap backing store for the global allocator.
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Initialize the heap range used by the global allocator.
pub fn init_heap() {
    let start: usize = addr_of_mut!(HEAP) as *mut u8 as usize;
    let end: usize = start + HEAP_SIZE;

    HEAP_START.store(start, Ordering::SeqCst);
    HEAP_END.store(end, Ordering::SeqCst);
    NEXT.store(start, Ordering::SeqCst);
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
