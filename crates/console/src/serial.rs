use core::fmt::Write;
use spin::{Mutex, Once};

const COM1: u16 = 0x3F8;

pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    pub const fn new(base: u16) -> Self {
        Self { base }
    }

    pub fn init(&mut self) {
        outb(self.base + 1, 0x00); // Disable interrupts
        outb(self.base + 3, 0x80); // Enable DLAB
        outb(self.base + 0, 0x03); // Baud divisor low (38400)
        outb(self.base + 1, 0x00); // Baud divisor high
        outb(self.base + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(self.base + 2, 0xC7); // Enable FIFO, clear, 14-byte threshold
        outb(self.base + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }

    fn is_transmit_empty(&self) -> bool {
        inb(self.base + 5) & 0x20 != 0
    }

    fn write_byte(&mut self, byte: u8) {
        while !self.is_transmit_empty() {}
        outb(self.base, byte);
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

static SERIAL1: Once<Mutex<SerialPort>> = Once::new();

pub fn init_serial() {
    SERIAL1.call_once(|| Mutex::new(SerialPort::new(COM1)));
    if let Some(serial) = SERIAL1.get() {
        serial.lock().init();
    }
}

pub fn _print(args: core::fmt::Arguments) {
    if let Some(serial) = SERIAL1.get() {
        let _ = serial.lock().write_fmt(args);
    }
}

#[inline]
fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags),
        )
    };
}

#[inline]
fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags),
        )
    };
    value
}
