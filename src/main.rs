#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::panic::PanicInfo;
use kernal::terminal::{
    println,
    print,
    clear,
    error,
    set_position,
    set_background
};  

use kernal::vga::Color;
use kernal::terminal;
#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernal::init();
    kernal::post();
    kernal::enable_interrupts();
    kernal::set_tick_rate(1000);
    terminal::clear!();
    terminal::update_cursor();

    kernal::crash();

    loop {
        kernal::pause_for(1);
    }
}

#[panic_handler]
pub fn panic_handler(_info : &PanicInfo) -> ! {
    kernal::disable_interrupts();
    terminal::set_background!(Color::Red as u8);
    clear!();
    set_position!(0,0);
    error!("{}", _info.message().unwrap());
    loop {}
}