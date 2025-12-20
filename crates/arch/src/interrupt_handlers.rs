use crate::{idt::InterruptIndex, interrupts};
use console::serial_println;
use x86_64::{
    VirtAddr,
    addr::VirtAddrNotValid,
    instructions::{
        hlt,
        port::{Port, PortGeneric, ReadWriteAccess},
    },
    registers::control::Cr2,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
};

fn halt_loop() -> ! {
    loop {
        hlt();
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    interrupts::end_of_interrupt(InterruptIndex::Timer);
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port: PortGeneric<u8, ReadWriteAccess> = Port::<u8>::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    keyboard::on_scancode(scancode);
    interrupts::end_of_interrupt(InterruptIndex::Keyboard);
}

pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    halt_loop();
}

pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    serial_println!(
        "EXCEPTION: GENERAL PROTECTION FAULT (code={:#x})\n{:#?}",
        error_code,
        stack_frame
    );
    halt_loop();
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let addr: Result<VirtAddr, VirtAddrNotValid> = Cr2::read();
    serial_println!(
        "EXCEPTION: PAGE FAULT\naddr={:#x} error={:?}\n{:#?}",
        addr.expect("why").as_u64(),
        error_code,
        stack_frame
    );
    halt_loop();
}
