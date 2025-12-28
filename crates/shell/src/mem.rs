use console::console_trait::ConsoleOut;
use core::fmt::Write;
use memory::MemRegion;

pub fn show_memory_map<C, I>(console: &mut C, regions: I)
where
    C: ConsoleOut + Write,
    I: IntoIterator<Item = MemRegion>,
{
    memory::dump_memory_map(regions, console);
}

pub fn alloc_frame<C, I>(console: &mut C, regions: I) -> Option<u64>
where
    C: ConsoleOut + Write,
    I: IntoIterator<Item = MemRegion>,
{
    memory::alloc_frame(regions, console)
}
