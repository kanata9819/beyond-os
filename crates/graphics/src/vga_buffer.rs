const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const COLOR: u8 = 0x0f;

pub fn print(s: &str) {
    for (i, byte) in s.bytes().enumerate() {
        unsafe {
            *VGA_BUFFER.add(i * 2) = byte;
            *VGA_BUFFER.add(i * 2 + 1) = COLOR;
        }
    }
}
