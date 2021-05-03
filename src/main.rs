#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernal::terminal::{
    println,
    print,
    clear,
    error,
    set_position
};  

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernal::init();
    kernal::enable_interrupts();
    let mut i:u128 = 0;
    kernal::spin!();
    loop {/*println!("Hello #{}",i); i += 1*/}
}

#[panic_handler]
pub fn panic_handler(_info : &PanicInfo) -> ! {
    clear!();
    set_position!(0,0);
    error!("{}", _info);
    loop {}
}