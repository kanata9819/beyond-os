use core::fmt::Write;
use spin::{Mutex, Once};

/// Base I/O port address for COM1.
const COM1: u16 = 0x3F8;

/// Minimal COM port wrapper for serial output.
pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    /// Create a serial port object with the given base I/O address.
    pub const fn new(base: u16) -> Self {
        Self { base }
    }

    /// Initialize the UART for 8N1 at 38400 baud and enable FIFO.
    ///
    /// Register writes (base = COM1):
    /// - base+1 (IER): set to 0x00 to disable UART interrupts during setup.
    /// - base+3 (LCR): set DLAB (0x80) to access baud divisor registers.
    /// - base+0 (DLL): set divisor low byte to 0x03.
    /// - base+1 (DLM): set divisor high byte to 0x00.
    ///   Divisor 0x0003 => 115200 / 3 = 38400 baud.
    /// - base+3 (LCR): set to 0x03 for 8 data bits, no parity, 1 stop bit.
    /// - base+2 (FCR): set to 0xC7 to enable FIFO, clear buffers, 14-byte threshold.
    /// - base+4 (MCR): set to 0x0B to enable IRQs and assert RTS/DSR.
    pub fn init(&mut self) {
        outb(self.base + 1, 0x00); // Disable interrupts
        outb(self.base + 3, 0x80); // Enable DLAB
        outb(self.base + 0, 0x03); // Baud divisor low (38400)
        outb(self.base + 1, 0x00); // Baud divisor high
        outb(self.base + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(self.base + 2, 0xC7); // Enable FIFO, clear, 14-byte threshold
        outb(self.base + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }

    /// Check the transmit buffer empty flag.
    fn is_transmit_empty(&self) -> bool {
        inb(self.base + 5) & 0x20 != 0
    }

    /// Write one byte, waiting until the UART is ready.
    fn write_byte(&mut self, byte: u8) {
        while !self.is_transmit_empty() {}
        outb(self.base, byte);
    }
}

impl Write for SerialPort {
    /// Write a UTF-8 string over the serial port.
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

/// Global COM1 instance initialized on first use.
static SERIAL1: Once<Mutex<SerialPort>> = Once::new();

/// Initialize the global COM1 port.
pub fn init_serial() {
    SERIAL1.call_once(|| Mutex::new(SerialPort::new(COM1)));
    if let Some(serial) = SERIAL1.get() {
        serial.lock().init();
    }
}

/// Core printer used by the `serial_print!` macros.
pub fn _print(args: core::fmt::Arguments) {
    if let Some(serial) = SERIAL1.get() {
        let _ = serial.lock().write_fmt(args);
    }
}

#[inline]
/// Write an 8-bit value to an I/O port using the x86 `out` instruction.
///
/// `port` is loaded into the DX register, `value` into AL, then `out dx, al`
/// sends the byte to the hardware device mapped at that I/O port.
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
/// Read an 8-bit value from an I/O port using the x86 `in` instruction.
///
/// `port` is loaded into DX, `in al, dx` reads one byte from the device, and
/// the result is returned from AL as the function result.
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
