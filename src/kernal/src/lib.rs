#![no_std]
#![feature(decl_macro)]
#![feature(abi_x86_interrupt)]
pub mod vga;
pub mod terminal;
pub mod interrupts;
pub mod gdt;
pub mod pics;
pub mod pit;
pub mod keyboard;
pub mod serial;

pub fn post() {
    serial::print!("Running POST...");
    serial::println!("[OK]");
}

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

pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}

pub macro breakpoint() {
    x86_64::instructions::interrupts::int3();
}

pub macro spin() {
    loop {x86_64::instructions::hlt();}
}

pub fn wait_for_interrupt() { x86_64::instructions::hlt(); }

pub fn pause_for(ticks : usize) {
    for _ in 0..=ticks {
        x86_64::instructions::hlt();
    }
}

pub fn set_tick_rate(hertz : usize) {
    let mut reload_value : u16 = ((pit::FREQUENCY / hertz) & 0xFFFF) as u16;
    if reload_value < 18 {reload_value = 18;}
    unsafe {
        pit::set_reload_value(reload_value);
    }
}

pub macro post_fail($code:expr) {
    fail_post($code);
}

fn fail_post(code : usize) {
    panic!("Post Failed [EC: [0x{:04x}]", code);
}


pub static POST_FAIL_TERMINAL : usize = 0x0100;