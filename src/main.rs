#![no_std]
#![no_main]
#![feature(decl_macro)]
#![feature(abi_x86_interrupt)]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

use core::panic::PanicInfo;

use drivers::vga::ColorCode;
use drivers::vga::Color;
use terminal::{clear, set_cell, set_color, reset_cursor, get_char};
use terminal::{set_background, set_foreground, get_fg_color, get_bg_color};

mod drivers;
mod terminal;
mod traits;


#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_print();
    test_colors();

    terminal::set_position(0,24);
    terminal::set_foreground!(Color::Yellow);
    print!("[{}@{}]>> ","","");
    loop {}
}

//Panic Handler
#[panic_handler]
pub fn panic_handler(info:&PanicInfo) -> ! {
    set_color!(ColorCode::new(Color::White, Color::Blue));
    clear!();
    println!();
    println!("==== KERNAL PANIC ====");
    println!(" {}", info);

    for x in 0..80 { set_cell!(x , 0 , ColorCode::new(Color::White, Color::Cyan)) }
    for x in 0..25 { set_cell!(0 , x , ColorCode::new(Color::White, Color::Cyan)) }
    for x in 0..25 { set_cell!(79, x , ColorCode::new(Color::White, Color::Cyan)) }
    for x in 0..80 { set_cell!(x , 24, ColorCode::new(Color::White, Color::Cyan)) }

    serial_println!("{}", info);
    loop {}
}


pub fn test_print() {
    clear!();
    reset_cursor!();
    println!("[test_print] - Hello World!");
    if get_char!(0,0) != b'[' {
        panic!("Printing Has Failed!");
    }
    serial_println!("Passed test_print");
}


pub fn test_colors() {
    clear!();
    reset_cursor!();
    set_foreground!(Color::White);
    set_background!(Color::Blue);
    print!(" ");
    if get_fg_color!(0,0) != Color::White {
        panic!("Didn't Set FG Color Correctly! Got: {:?}", get_fg_color!(0,0))
    }
    if get_bg_color!(0,0) != Color::Blue {
        panic!("Didn't Set BG Color Correctly! Got: {:?}", get_fg_color!(0,0))
    }

    serial_println!("Passed test_colors");
}

