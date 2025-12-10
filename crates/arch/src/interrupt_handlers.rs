use crate::idt::InterruptIndex;
use crate::pic::PICS;
use spin::Mutex;
use x86_64::{
    instructions::port::{Port, PortGeneric, ReadWriteAccess},
    structures::idt::InterruptStackFrame,
};

// ===== 割り込みハンドラたち =====
pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

static KEYBOARD_BUFFER: Mutex<keyboard::KeyboardBuffer> =
    Mutex::new(keyboard::KeyboardBuffer::new());

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // 0x60 からスキャンコードを読む
    let mut port: PortGeneric<u8, ReadWriteAccess> = Port::<u8>::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // バッファに詰める
    KEYBOARD_BUFFER.lock().push(scancode);

    // PIC に EOI
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
