#![no_std]
#![feature(decl_macro)]
#![feature(abi_x86_interrupt)]
mod vga;
pub mod terminal;
pub mod interrupts;
pub mod gdt;

pub fn init() {
    gdt::init_gdt();
    interrupts::init_idt();
}