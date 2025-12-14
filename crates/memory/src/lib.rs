#[derive(Clone, Copy)]
pub struct MemRegion {
    pub start: u64,
    pub end: u64,
    pub kind: MemRegionKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemRegionKind {
    Usable,
    Reserved,
    Other,
}

pub fn dump_memory_map<I>(regions: I, console: &mut impl core::fmt::Write)
where
    I: IntoIterator<Item = MemRegion>,
{
    let mut usable_bytes: u64 = 0;

    writeln!(console, "== memory map ==").unwrap();

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
