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

const CONFIG_ENABLE: u32 = 0x8000_0000;
const BUS_SHIFT: u32 = 16;
const DEVICE_SHIFT: u32 = 11;
const FUNCTION_SHIFT: u32 = 8;
const OFFSET_ALIGN_MASK: u32 = 0xfc;

const COMMAND_STATUS_OFFSET: u8 = 0x04;
const CLASS_FIELDS_OFFSET: u8 = 0x08;
const HEADER_TYPE_OFFSET: u8 = 0x0c;
const BAR0_OFFSET: u8 = 0x10;

const VENDOR_ID_OFFSET: u8 = 0x00;
const DEVICE_ID_OFFSET: u8 = 0x02;

const HEADER_TYPE_MASK: u32 = 0xff;
const MULTIFUNCTION_BIT: u8 = 0x80;

const CLASS_SHIFT: u32 = 24;
const SUBCLASS_SHIFT: u32 = 16;
const PROG_IF_SHIFT: u32 = 8;

const WORD_SELECT_MASK: u8 = 0x2;
const WORD_MASK: u32 = 0xffff;
const WORD_SHIFT_STEP: u8 = 8;

const STATUS_MASK: u32 = 0xffff_0000;
const COMMAND_IO_SPACE: u16 = 0x0001;
const COMMAND_BUS_MASTER: u16 = 0x0004;

const MAX_BUS: u16 = 255;
const DEVICE_COUNT: u8 = 32;
const FUNCTION_COUNT_MULTIFUNCTION: u8 = 8;
const FUNCTION_COUNT_SINGLE: u8 = 1;
const INVALID_VENDOR_ID: u16 = 0xffff;

const BAR_UPPER_OFFSET: u8 = 4;
const LAST_BAR_INDEX: u8 = 5;

const BAR_COUNT: u8 = 6;
const BAR_STRIDE: u8 = 4;
const BAR_SIZE_PROBE_VALUE: u32 = 0xffff_ffff;

const BAR_IO_SPACE_BIT: u32 = 0x1;
const BAR_IO_BASE_MASK: u32 = 0xffff_fffc;

const BAR_MEM_TYPE_MASK: u32 = 0x3;
const BAR_MEM_TYPE_64: u32 = 0x2;
const BAR_MEM_BASE_MASK: u32 = 0xffff_fff0;
const BAR_MEM_TYPE_SHIFT: u32 = 1;
const BAR_UNIMPLEMENTED_VALUE: u32 = 0;
const SIZE_MASK_INCREMENT_U32: u32 = 1;
const SIZE_MASK_INCREMENT_U64: u64 = 1;
const U64_UPPER_SHIFT: u32 = 32;

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
    let address = CONFIG_ENABLE
        // Encode bus in bits 23..16.
        | ((bus as u32) << BUS_SHIFT)
        // Encode device in bits 15..11.
        | ((device as u32) << DEVICE_SHIFT)
        // Encode function in bits 10..8.
        | ((function as u32) << FUNCTION_SHIFT)
        // Align offset to a 32-bit boundary (bits 7..2).
        | (offset as u32 & OFFSET_ALIGN_MASK);

    unsafe {
        let mut addr_port = Port::<u32>::new(CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(CONFIG_DATA);
        addr_port.write(address);
        data_port.read()
    }
}

/// Write a 32-bit value into PCI config space via 0xCF8/0xCFC.
fn write_config_dword(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = CONFIG_ENABLE
        | ((bus as u32) << BUS_SHIFT)
        | ((device as u32) << DEVICE_SHIFT)
        | ((function as u32) << FUNCTION_SHIFT)
        | (offset as u32 & OFFSET_ALIGN_MASK);

    unsafe {
        let mut addr_port = Port::<u32>::new(CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(CONFIG_DATA);
        addr_port.write(address);
        data_port.write(value);
    }
}

/// Enable PCI command bits (I/O space and bus mastering).
pub fn enable_io_bus_master(bus: u8, device: u8, function: u8) {
    let value = read_config_dword(bus, device, function, COMMAND_STATUS_OFFSET);
    let cmd = (value & WORD_MASK) as u16;
    let status = value & STATUS_MASK;
    let new_cmd = cmd | COMMAND_IO_SPACE | COMMAND_BUS_MASTER;
    write_config_dword(
        bus,
        device,
        function,
        COMMAND_STATUS_OFFSET,
        status | (new_cmd as u32),
    );
}

/// Read a 16-bit value from PCI config space.
/// Reads the containing dword, then selects lower/upper 16 bits by offset bit 1.
fn read_config_word(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    // Read the containing dword, then extract a 16-bit half.
    let value = read_config_dword(bus, device, function, offset & OFFSET_ALIGN_MASK as u8);
    // If offset bit 1 is set, choose the upper 16 bits (shift by 16).
    let shift = (offset & WORD_SELECT_MASK) * WORD_SHIFT_STEP;
    ((value >> shift) & WORD_MASK) as u16
}

/// Read class/subclass/prog-if/revision from offset 0x08.
/// The dword layout is [class][subclass][prog-if][revision].
fn read_class_fields(bus: u8, device: u8, function: u8) -> (u8, u8, u8, u8) {
    // Offset 0x08 packs class/subclass/prog-if/revision into one dword.
    let value = read_config_dword(bus, device, function, CLASS_FIELDS_OFFSET);
    let class_code = (value >> CLASS_SHIFT) as u8;
    let subclass = (value >> SUBCLASS_SHIFT) as u8;
    let prog_if = (value >> PROG_IF_SHIFT) as u8;
    let revision_id = value as u8;
    (class_code, subclass, prog_if, revision_id)
}

/// Read the header_type from offset 0x0C (used for multi-function detection).
fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    // Offset 0x0C contains the header type in bits 23..16.
    let value = read_config_dword(bus, device, function, HEADER_TYPE_OFFSET);
    ((value >> SUBCLASS_SHIFT) & HEADER_TYPE_MASK) as u8
}

/// Scan all buses/devices/functions and pass each present device to the callback.
/// Probe function 0; if header_type bit7 is set, scan functions 0..7.
pub fn scan(mut f: impl FnMut(PciDevice)) {
    for bus in 0u16..=MAX_BUS {
        for device in 0u8..DEVICE_COUNT {
            let vendor_id = read_config_word(bus as u8, device, 0, VENDOR_ID_OFFSET);
            if vendor_id == INVALID_VENDOR_ID {
                continue;
            }

            let header_type = read_header_type(bus as u8, device, 0);
            let functions = if header_type & MULTIFUNCTION_BIT != 0 {
                FUNCTION_COUNT_MULTIFUNCTION
            } else {
                FUNCTION_COUNT_SINGLE
            };

            for function in 0u8..functions {
                let vendor_id = read_config_word(bus as u8, device, function, VENDOR_ID_OFFSET);
                if vendor_id == INVALID_VENDOR_ID {
                    continue;
                }

                let device_id = read_config_word(bus as u8, device, function, DEVICE_ID_OFFSET);
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
    if index >= BAR_COUNT {
        return None;
    }

    let offset = BAR0_OFFSET + (index * BAR_STRIDE);
    let original = read_config_dword(bus, device, function, offset);
    if original == BAR_UNIMPLEMENTED_VALUE {
        return None;
    }

    // I/O BAR: bit0 = 1, base is in bits 31..2.
    if (original & BAR_IO_SPACE_BIT) == BAR_IO_SPACE_BIT {
        let base = (original & BAR_IO_BASE_MASK) as u64;
        let size = bar_size_io(bus, device, function, offset, original);
        return Some(Bar {
            kind: BarKind::Io,
            base,
            size,
        });
    }

    // Memory BAR: bit0 = 0, type is in bits 2..1.
    let mem_type = (original >> BAR_MEM_TYPE_SHIFT) & BAR_MEM_TYPE_MASK;
    if mem_type == BAR_MEM_TYPE_64 {
        // 64-bit MMIO uses the next BAR as the upper 32 bits.
        if index == LAST_BAR_INDEX {
            return None;
        }
        let upper = read_config_dword(bus, device, function, offset + BAR_UPPER_OFFSET);
        let base = ((upper as u64) << U64_UPPER_SHIFT) | ((original & BAR_MEM_BASE_MASK) as u64);
        let size = bar_size_mmio64(bus, device, function, offset, original, upper);
        return Some(Bar {
            kind: BarKind::Mmio64,
            base,
            size,
        });
    }

    // 32-bit MMIO.
    let base = (original & BAR_MEM_BASE_MASK) as u64;
    let size = bar_size_mmio32(bus, device, function, offset, original);
    Some(Bar {
        kind: BarKind::Mmio32,
        base,
        size,
    })
}

fn bar_size_io(bus: u8, device: u8, function: u8, offset: u8, original: u32) -> Option<u64> {
    // Write all 1s, read back the size mask, then restore original value.
    write_config_dword(bus, device, function, offset, BAR_SIZE_PROBE_VALUE);
    let mask = read_config_dword(bus, device, function, offset) & BAR_IO_BASE_MASK;
    write_config_dword(bus, device, function, offset, original);
    let size = (!mask).wrapping_add(SIZE_MASK_INCREMENT_U32) & BAR_IO_BASE_MASK;
    if size == 0 { None } else { Some(size as u64) }
}

fn bar_size_mmio32(bus: u8, device: u8, function: u8, offset: u8, original: u32) -> Option<u64> {
    // Write all 1s, read back the size mask, then restore original value.
    write_config_dword(bus, device, function, offset, BAR_SIZE_PROBE_VALUE);
    let mask = read_config_dword(bus, device, function, offset) & BAR_MEM_BASE_MASK;
    write_config_dword(bus, device, function, offset, original);
    let size = (!mask).wrapping_add(SIZE_MASK_INCREMENT_U32) & BAR_MEM_BASE_MASK;
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
    write_config_dword(bus, device, function, offset, BAR_SIZE_PROBE_VALUE);
    write_config_dword(
        bus,
        device,
        function,
        offset + BAR_UPPER_OFFSET,
        BAR_SIZE_PROBE_VALUE,
    );
    let low_mask = read_config_dword(bus, device, function, offset) & BAR_MEM_BASE_MASK;
    let high_mask = read_config_dword(bus, device, function, offset + BAR_UPPER_OFFSET);
    write_config_dword(bus, device, function, offset, original_low);
    write_config_dword(
        bus,
        device,
        function,
        offset + BAR_UPPER_OFFSET,
        original_high,
    );
    let mask = ((high_mask as u64) << U64_UPPER_SHIFT) | (low_mask as u64);
    let size = (!mask).wrapping_add(SIZE_MASK_INCREMENT_U64);
    if size == 0 { None } else { Some(size) }
}
