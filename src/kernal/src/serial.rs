use core::fmt;
use core::fmt::Write;

pub fn _print(args : fmt::Arguments) {
    SERIAL1.lock().write_fmt(args).expect("Unable To Write To the Serial Port");
}

pub macro print($($arg:tt)*) {
    crate::serial::_print(format_args!($($arg)*));
}

pub macro println($($arg:tt)*) {
    crate::serial::print!("{}\r\n", format_args!($($arg)*));
}

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}