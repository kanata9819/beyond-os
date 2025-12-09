use pic8259::ChainedPics;
use spin::{Mutex, Once};
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// PIC を 32 番以降にリマップ
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // IRQ0
    Keyboard,             // IRQ1 = 33
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

// ===== IDT 本体 =====
static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn init_idt() {
    let mut idt: InterruptDescriptorTable = InterruptDescriptorTable::new();

    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);

    let idt_ref: &InterruptDescriptorTable = IDT.call_once(|| idt);
    idt_ref.load();
}

// ===== PIC (8259) =====
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init_pics() {
    unsafe { PICS.lock().initialize() };
}

// ===== キーボード用リングバッファ =====
const KB_BUF_SIZE: usize = 256;

struct KeyboardBuffer {
    buf: [u8; KB_BUF_SIZE],
    head: usize,
    tail: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        Self {
            buf: [0; KB_BUF_SIZE],
            head: 0,
            tail: 0,
        }
    }

    fn push(&mut self, scancode: u8) {
        let next_head = (self.head + 1) % KB_BUF_SIZE;
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
            let val = self.buf[self.tail];
            self.tail = (self.tail + 1) % KB_BUF_SIZE;
            Some(val)
        }
    }
}

// 割り込みハンドラと main で共有するバッファ
static KEYBOARD_BUFFER: Mutex<KeyboardBuffer> = Mutex::new(KeyboardBuffer::new());

// main から呼び出す用の関数
pub fn pop_scancode() -> Option<u8> {
    KEYBOARD_BUFFER.lock().pop()
}

// ===== 割り込みハンドラたち =====
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // 0x60 からスキャンコードを読む
    let mut port = Port::<u8>::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // バッファに詰める
    KEYBOARD_BUFFER.lock().push(scancode);

    // PIC に EOI
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
