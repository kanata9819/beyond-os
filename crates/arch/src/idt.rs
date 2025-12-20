use crate::interrupt_handlers::{
    general_protection_fault_handler, invalid_opcode_handler, keyboard_interrupt_handler,
    page_fault_handler, timer_interrupt_handler,
};
use crate::pic::PIC_1_OFFSET;
use spin::once::Once;
use x86_64::structures::idt::InterruptDescriptorTable;

static IDT: Once<InterruptDescriptorTable> = Once::new();

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

pub fn init_idt() {
    let mut idt: InterruptDescriptorTable = InterruptDescriptorTable::new();
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);

    let idt_ref: &InterruptDescriptorTable = IDT.call_once(|| idt);
    idt_ref.load();
}
