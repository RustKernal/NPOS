#![no_std]
#![feature(decl_macro)]
#![feature(abi_x86_interrupt)]
mod vga;
pub mod terminal;
pub mod interrupts;
pub mod gdt;
pub mod pics;
pub mod pit;

pub fn init() {
    gdt::init_gdt();
    interrupts::init_idt();
    unsafe {
        pics::PICS.lock().initialize();
    }
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

pub macro breakpoint() {
    x86_64::instructions::interrupts::int3();
}

pub macro spin() {
    loop {x86_64::instructions::hlt();}
}

pub fn wait_for_interrupt() { x86_64::instructions::hlt(); }

pub fn pause_for(ticks : u8) {
    for _ in 0..=ticks {
        x86_64::instructions::hlt();
    }
}

pub fn set_tick_rate(hertz : usize) {

}