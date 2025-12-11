#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

const KB_BUF_SIZE: usize = 256;
static KEYBOARD_BUFFER: Mutex<KeyboardBuffer> = Mutex::new(KeyboardBuffer::new());

pub fn on_scancode(scancode: u8) {
    KEYBOARD_BUFFER.lock().push(scancode);
}

pub fn pop_scancode() -> Option<u8> {
    KEYBOARD_BUFFER.lock().pop()
}

pub struct KeyboardBuffer {
    buf: [u8; KB_BUF_SIZE],
    head: usize,
    tail: usize,
}

impl KeyboardBuffer {
    pub const fn new() -> Self {
        Self {
            buf: [0; KB_BUF_SIZE],
            head: 0,
            tail: 0,
        }
    }

    fn push(&mut self, scancode: u8) {
        let next_head: usize = (self.head + 1) % KB_BUF_SIZE;
        // 一杯のときは上書き or 無視、好きな方で
        if next_head != self.tail {
            self.buf[self.head] = scancode;
            self.head = next_head;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail {
            None
        } else {
            let val: u8 = self.buf[self.tail];
            self.tail = (self.tail + 1) % KB_BUF_SIZE;
            Some(val)
        }
    }
}

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

// Shift状態を覚えて大文字や記号キーの入力も返す
static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);

pub fn scancode_to_char(sc: u8) -> Option<char> {
    const LEFT_SHIFT: u8 = 0x2A;
    const RIGHT_SHIFT: u8 = 0x36;
    const RELEASE_MASK: u8 = 0x80;

    match sc {
        LEFT_SHIFT | RIGHT_SHIFT => {
            SHIFT_PRESSED.store(true, Ordering::Relaxed);
            return None;
        }
        sc if sc == (LEFT_SHIFT | RELEASE_MASK) || sc == (RIGHT_SHIFT | RELEASE_MASK) => {
            SHIFT_PRESSED.store(false, Ordering::Relaxed);
            return None;
        }
        _ => {}
    }

    let shifted: bool = SHIFT_PRESSED.load(Ordering::Relaxed);

    match (shifted, sc) {
        // 数字キー
        (false, 0x02) => Some('1'),
        (false, 0x03) => Some('2'),
        (false, 0x04) => Some('3'),
        (false, 0x05) => Some('4'),
        (false, 0x06) => Some('5'),
        (false, 0x07) => Some('6'),
        (false, 0x08) => Some('7'),
        (false, 0x09) => Some('8'),
        (false, 0x0A) => Some('9'),
        (false, 0x0B) => Some('0'),

        // 数字キーのShift記号
        (true, 0x02) => Some('!'),
        (true, 0x03) => Some('@'),
        (true, 0x04) => Some('#'),
        (true, 0x05) => Some('$'),
        (true, 0x06) => Some('%'),
        (true, 0x07) => Some('^'),
        (true, 0x08) => Some('&'),
        (true, 0x09) => Some('*'),
        (true, 0x0A) => Some('('),
        (true, 0x0B) => Some(')'),
        (false, 0x0C) => Some('-'),
        (true, 0x0C) => Some('_'),
        (false, 0x0D) => Some('='),
        (true, 0x0D) => Some('+'),

        // アルファベット
        (false, 0x10) => Some('q'),
        (false, 0x11) => Some('w'),
        (false, 0x12) => Some('e'),
        (false, 0x13) => Some('r'),
        (false, 0x14) => Some('t'),
        (false, 0x15) => Some('y'),
        (false, 0x16) => Some('u'),
        (false, 0x17) => Some('i'),
        (false, 0x18) => Some('o'),
        (false, 0x19) => Some('p'),
        (true, 0x10) => Some('Q'),
        (true, 0x11) => Some('W'),
        (true, 0x12) => Some('E'),
        (true, 0x13) => Some('R'),
        (true, 0x14) => Some('T'),
        (true, 0x15) => Some('Y'),
        (true, 0x16) => Some('U'),
        (true, 0x17) => Some('I'),
        (true, 0x18) => Some('O'),
        (true, 0x19) => Some('P'),

        (false, 0x1E) => Some('a'),
        (false, 0x1F) => Some('s'),
        (false, 0x20) => Some('d'),
        (false, 0x21) => Some('f'),
        (false, 0x22) => Some('g'),
        (false, 0x23) => Some('h'),
        (false, 0x24) => Some('j'),
        (false, 0x25) => Some('k'),
        (false, 0x26) => Some('l'),
        (true, 0x1E) => Some('A'),
        (true, 0x1F) => Some('S'),
        (true, 0x20) => Some('D'),
        (true, 0x21) => Some('F'),
        (true, 0x22) => Some('G'),
        (true, 0x23) => Some('H'),
        (true, 0x24) => Some('J'),
        (true, 0x25) => Some('K'),
        (true, 0x26) => Some('L'),

        (false, 0x2C) => Some('z'),
        (false, 0x2D) => Some('x'),
        (false, 0x2E) => Some('c'),
        (false, 0x2F) => Some('v'),
        (false, 0x30) => Some('b'),
        (false, 0x31) => Some('n'),
        (false, 0x32) => Some('m'),
        (true, 0x2C) => Some('Z'),
        (true, 0x2D) => Some('X'),
        (true, 0x2E) => Some('C'),
        (true, 0x2F) => Some('V'),
        (true, 0x30) => Some('B'),
        (true, 0x31) => Some('N'),
        (true, 0x32) => Some('M'),

        // 記号キー
        (false, 0x1A) => Some('['),
        (true, 0x1A) => Some('{'),
        (false, 0x1B) => Some(']'),
        (true, 0x1B) => Some('}'),
        (false, 0x27) => Some(';'),
        (true, 0x27) => Some(':'),
        (false, 0x28) => Some('\''),
        (true, 0x28) => Some('"'),
        (false, 0x29) => Some('`'),
        (true, 0x29) => Some('~'),
        (false, 0x2B) => Some('\\'),
        (true, 0x2B) => Some('|'),
        (false, 0x33) => Some(','),
        (true, 0x33) => Some('<'),
        (false, 0x34) => Some('.'),
        (true, 0x34) => Some('>'),
        (false, 0x35) => Some('/'),
        (true, 0x35) => Some('?'),

        // 制御キー
        (_, 0x0E) => Some('\u{0008}'), // Backspace
        (_, 0x0F) => Some('\t'),       // Tab
        (_, 0x1C) => Some('\n'),       // Enter
        (_, 0x39) => Some(' '),        // Space

        _ => None,
    }
}
