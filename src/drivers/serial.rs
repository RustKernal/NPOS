use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;


#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    COMM0.lock().write_fmt(args).expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\r\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\r\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\r\n"), $($arg)*));
}

lazy_static! {
    pub static ref COMM0: Mutex<SerialPort> = {
        let mut port = unsafe {SerialPort::new(0x3F8)};
        port.init();
        Mutex::new(port)
    };
}



pub struct _SerialPrinter {
    port : COMM0
}

