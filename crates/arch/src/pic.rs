use pic8259::ChainedPics;
use spin::{Mutex, Once};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub trait InterruptController {
    fn init(&self);
    fn end_of_interrupt(&self, irq: crate::idt::InterruptIndex);
}

pub struct Pic8259Controller {
    pics: Mutex<ChainedPics>,
}

impl InterruptController for Pic8259Controller {
    fn init(&self) {
        unsafe {
            self.pics.lock().initialize();
        }
    }

    fn end_of_interrupt(&self, irq: crate::idt::InterruptIndex) {
        unsafe {
            self.pics.lock().notify_end_of_interrupt(irq.as_u8());
        }
    }
}

impl Pic8259Controller {
    fn new() -> Self {
        Self {
            pics: Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }),
        }
    }

    pub fn pic_end_of_interrupt(irq: crate::idt::InterruptIndex) {
        PIC8259_CONTROLLER
            .get()
            .expect("PIC not initialized")
            .end_of_interrupt(irq);
    }
}

static PIC8259_CONTROLLER: Once<Pic8259Controller> = Once::new();

pub fn pic_controller() -> &'static (dyn InterruptController + Sync) {
    PIC8259_CONTROLLER.call_once(Pic8259Controller::new);
    PIC8259_CONTROLLER.get().unwrap()
}
