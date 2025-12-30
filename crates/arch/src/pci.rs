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

fn read_config_dword(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = 0x8000_0000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | (offset as u32 & 0xfc);

    unsafe {
        let mut addr_port = Port::<u32>::new(CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(CONFIG_DATA);
        addr_port.write(address);
        data_port.read()
    }
}

fn read_config_word(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let value = read_config_dword(bus, device, function, offset & 0xfc);
    let shift = (offset & 0x2) * 8;
    ((value >> shift) & 0xffff) as u16
}

fn read_class_fields(bus: u8, device: u8, function: u8) -> (u8, u8, u8, u8) {
    let value = read_config_dword(bus, device, function, 0x08);
    let class_code = (value >> 24) as u8;
    let subclass = (value >> 16) as u8;
    let prog_if = (value >> 8) as u8;
    let revision_id = value as u8;
    (class_code, subclass, prog_if, revision_id)
}

fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    let value = read_config_dword(bus, device, function, 0x0c);
    ((value >> 16) & 0xff) as u8
}

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
