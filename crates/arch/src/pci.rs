//! PCI quick notes (minimal)
//! - PCI devices are identified by Bus/Device/Function (BDF).
//! - Config space can be accessed via Configuration Mechanism #1 using I/O ports:
//!   - 0xCF8: CONFIG_ADDRESS (selects BDF and register offset)
//!   - 0xCFC: CONFIG_DATA (returns the 32-bit data)
//! - The 0xCF8 address has the following layout:
//!   - bit31: Enable (1 = valid access)
//!   - bit23..16: bus
//!   - bit15..11: device
//!   - bit10..8 : function
//!   - bit7..2  : register offset (dword aligned)
//! - Probe function 0 to detect presence (vendor_id == 0xFFFF means no device).
//! - If header_type bit7 is set, the device is multi-function; scan 0..7.
//! - class/subclass/prog-if/revision are packed into the dword at offset 0x08.
//! - virtio devices use vendor_id 0x1AF4.
use x86_64::instructions::port::Port;

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision_id: u8,
    pub header_type: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarKind {
    Io,
    Mmio32,
    Mmio64,
}

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub kind: BarKind,
    pub base: u64,
    pub size: Option<u64>,
}

/// Read a 32-bit value from PCI config space via 0xCF8/0xCFC.
/// Encodes BDF and a dword-aligned offset into 0xCF8, then reads from 0xCFC.
fn read_config_dword(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    // 0x8000_0000 sets the "enable" bit for config space access.
    let address = 0x8000_0000u32
        // Encode bus in bits 23..16.
        | ((bus as u32) << 16)
        // Encode device in bits 15..11.
        | ((device as u32) << 11)
        // Encode function in bits 10..8.
        | ((function as u32) << 8)
        // Align offset to a 32-bit boundary (bits 7..2).
        | (offset as u32 & 0xfc);

    unsafe {
        let mut addr_port = Port::<u32>::new(CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(CONFIG_DATA);
        addr_port.write(address);
        data_port.read()
    }
}

/// Write a 32-bit value into PCI config space via 0xCF8/0xCFC.
fn write_config_dword(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = 0x8000_0000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | (offset as u32 & 0xfc);

    unsafe {
        let mut addr_port = Port::<u32>::new(CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(CONFIG_DATA);
        addr_port.write(address);
        data_port.write(value);
    }
}

/// Read a 16-bit value from PCI config space.
/// Reads the containing dword, then selects lower/upper 16 bits by offset bit 1.
fn read_config_word(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    // Read the containing dword, then extract a 16-bit half.
    let value = read_config_dword(bus, device, function, offset & 0xfc);
    // If offset bit 1 is set, choose the upper 16 bits (shift by 16).
    let shift = (offset & 0x2) * 8;
    ((value >> shift) & 0xffff) as u16
}

/// Read class/subclass/prog-if/revision from offset 0x08.
/// The dword layout is [class][subclass][prog-if][revision].
fn read_class_fields(bus: u8, device: u8, function: u8) -> (u8, u8, u8, u8) {
    // Offset 0x08 packs class/subclass/prog-if/revision into one dword.
    let value = read_config_dword(bus, device, function, 0x08);
    let class_code = (value >> 24) as u8;
    let subclass = (value >> 16) as u8;
    let prog_if = (value >> 8) as u8;
    let revision_id = value as u8;
    (class_code, subclass, prog_if, revision_id)
}

/// Read the header_type from offset 0x0C (used for multi-function detection).
fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    // Offset 0x0C contains the header type in bits 23..16.
    let value = read_config_dword(bus, device, function, 0x0c);
    ((value >> 16) & 0xff) as u8
}

/// Scan all buses/devices/functions and pass each present device to the callback.
/// Probe function 0; if header_type bit7 is set, scan functions 0..7.
pub fn scan(mut f: impl FnMut(PciDevice)) {
    for bus in 0u16..=255 {
        for device in 0u8..32 {
            let vendor_id = read_config_word(bus as u8, device, 0, 0x00);
            if vendor_id == 0xffff {
                continue;
            }

            let header_type = read_header_type(bus as u8, device, 0);
            let functions = if header_type & 0x80 != 0 { 8 } else { 1 };

            for function in 0u8..functions {
                let vendor_id = read_config_word(bus as u8, device, function, 0x00);
                if vendor_id == 0xffff {
                    continue;
                }

                let device_id = read_config_word(bus as u8, device, function, 0x02);
                let (class_code, subclass, prog_if, revision_id) =
                    read_class_fields(bus as u8, device, function);
                let header_type = read_header_type(bus as u8, device, function);

                f(PciDevice {
                    bus: bus as u8,
                    device,
                    function,
                    vendor_id,
                    device_id,
                    class_code,
                    subclass,
                    prog_if,
                    revision_id,
                    header_type,
                });
            }
        }
    }
}

/// Read a BAR (0..5) and decode its type/base/size.
/// Returns None when the BAR is unimplemented or index is out of range.
pub fn read_bar(bus: u8, device: u8, function: u8, index: u8) -> Option<Bar> {
    if index >= 6 {
        return None;
    }

    let offset = 0x10u8 + (index * 4);
    let original = read_config_dword(bus, device, function, offset);
    if original == 0 {
        return None;
    }

    // I/O BAR: bit0 = 1, base is in bits 31..2.
    if (original & 0x1) == 0x1 {
        let base = (original & 0xffff_fffc) as u64;
        let size = bar_size_io(bus, device, function, offset, original);
        return Some(Bar {
            kind: BarKind::Io,
            base,
            size,
        });
    }

    // Memory BAR: bit0 = 0, type is in bits 2..1.
    let mem_type = (original >> 1) & 0x3;
    if mem_type == 0x2 {
        // 64-bit MMIO uses the next BAR as the upper 32 bits.
        if index == 5 {
            return None;
        }
        let upper = read_config_dword(bus, device, function, offset + 4);
        let base = ((upper as u64) << 32) | ((original & 0xffff_fff0) as u64);
        let size = bar_size_mmio64(bus, device, function, offset, original, upper);
        return Some(Bar {
            kind: BarKind::Mmio64,
            base,
            size,
        });
    }

    // 32-bit MMIO.
    let base = (original & 0xffff_fff0) as u64;
    let size = bar_size_mmio32(bus, device, function, offset, original);
    Some(Bar {
        kind: BarKind::Mmio32,
        base,
        size,
    })
}

fn bar_size_io(bus: u8, device: u8, function: u8, offset: u8, original: u32) -> Option<u64> {
    // Write all 1s, read back the size mask, then restore original value.
    write_config_dword(bus, device, function, offset, 0xffff_ffff);
    let mask = read_config_dword(bus, device, function, offset) & 0xffff_fffc;
    write_config_dword(bus, device, function, offset, original);
    let size = (!mask).wrapping_add(1) & 0xffff_fffc;
    if size == 0 { None } else { Some(size as u64) }
}

fn bar_size_mmio32(bus: u8, device: u8, function: u8, offset: u8, original: u32) -> Option<u64> {
    // Write all 1s, read back the size mask, then restore original value.
    write_config_dword(bus, device, function, offset, 0xffff_ffff);
    let mask = read_config_dword(bus, device, function, offset) & 0xffff_fff0;
    write_config_dword(bus, device, function, offset, original);
    let size = (!mask).wrapping_add(1) & 0xffff_fff0;
    if size == 0 { None } else { Some(size as u64) }
}

fn bar_size_mmio64(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
    original_low: u32,
    original_high: u32,
) -> Option<u64> {
    // Write all 1s into both halves, read masks, then restore originals.
    write_config_dword(bus, device, function, offset, 0xffff_ffff);
    write_config_dword(bus, device, function, offset + 4, 0xffff_ffff);
    let low_mask = read_config_dword(bus, device, function, offset) & 0xffff_fff0;
    let high_mask = read_config_dword(bus, device, function, offset + 4);
    write_config_dword(bus, device, function, offset, original_low);
    write_config_dword(bus, device, function, offset + 4, original_high);
    let mask = ((high_mask as u64) << 32) | (low_mask as u64);
    let size = (!mask).wrapping_add(1);
    if size == 0 { None } else { Some(size) }
}
