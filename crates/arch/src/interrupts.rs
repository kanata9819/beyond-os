use pic8259::ChainedPics;
use spin::Mutex;

// PIC を 32 番以降にリマップ
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub trait InterruptController {
    unsafe fn init(&self);
    unsafe fn end_of_interrupt(&self);
}

// ===== PIC (8259) =====
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init_pics() {
    unsafe { PICS.lock().initialize() };
}
