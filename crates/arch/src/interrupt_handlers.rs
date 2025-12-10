use crate::{idt::InterruptIndex, interrupts};
use spin::Mutex;
use x86_64::{
    instructions::port::{Port, PortGeneric, ReadWriteAccess},
    structures::idt::InterruptStackFrame,
};

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    interrupts::end_of_interrupt(InterruptIndex::Timer);
}

static KEYBOARD_BUFFER: Mutex<keyboard::KeyboardBuffer> =
    Mutex::new(keyboard::KeyboardBuffer::new());

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port: PortGeneric<u8, ReadWriteAccess> = Port::<u8>::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    KEYBOARD_BUFFER.lock().push(scancode);

    interrupts::end_of_interrupt(InterruptIndex::Keyboard);
}
