#![no_std]
#![no_main]

#[inline]
unsafe fn inb(port: u16) -> u8 {
    unsafe {
        let value: u8;
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags),
        );

        value
    }
}

pub unsafe fn read_scancode() -> Option<u8> {
    const KBD_STATUS_PORT: u16 = 0x64;
    const KBD_DATA_PORT: u16 = 0x60;

    unsafe {
        let status: u8 = inb(KBD_STATUS_PORT);

        if status & 0x01 != 0 {
            Some(inb(KBD_DATA_PORT))
        } else {
            None
        }
    }
}

pub fn scancode_to_char(sc: u8) -> Option<char> {
    match sc {
        // 数字 1〜0
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0A => Some('9'),
        0x0B => Some('0'),

        // アルファベット
        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),

        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),

        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),

        // Enter
        0x1C => Some('\n'),

        _ => None,
    }
}
