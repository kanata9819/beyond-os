use crate::idt::InterruptIndex;
use crate::pic::{self, InterruptController};
use spin::Once;

static CONTROLLER: Once<&'static (dyn InterruptController + Sync)> = Once::new();

pub fn init_interrupts() {
    // 今は PIC を採用
    CONTROLLER.call_once(|| pic::pic_controller());
    controller().init();
}

fn controller() -> &'static dyn InterruptController {
    *CONTROLLER
        .get()
        .expect("Interrupt controller not initialized")
}

pub fn end_of_interrupt(index: InterruptIndex) {
    controller().end_of_interrupt(index);
}
