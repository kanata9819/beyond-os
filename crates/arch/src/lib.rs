#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod idt;
pub mod interrupt_handler;
pub mod interrupts;
