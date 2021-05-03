#![no_std]
#![no_main]

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
    kernal::enable_interrupts();
    kernal::set_tick_rate(1000);
    terminal::clear!();
    loop {
        kernal::pause_for(1);
    }
}

#[panic_handler]
pub fn panic_handler(_info : &PanicInfo) -> ! {
    clear!();
    set_position!(0,0);
    error!("{}", _info);
    loop {}
}