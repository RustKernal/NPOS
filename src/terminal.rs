
use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{drivers::vga::{Color, ScreenChar, get_text_mode_buffer}, traits::text::Printer};


pub macro get_char($x:expr, $y:expr) {
    crate::terminal::_get_char($x, $y);
}

pub macro get_bg_color($x:expr, $y:expr) {
    crate::terminal::_get_bg_color($x, $y);
}

pub macro get_fg_color($x:expr, $y:expr) {
    crate::terminal::_get_fg_color($x, $y);
}

pub macro reset_cursor() {
    crate::terminal::_reset_cursor();
}

pub macro carriage_return() {
    crate::terminal::_carriage_return();
}

pub macro set_cursor($x:expr, $y:expr) {
    crate::terminal::_set_cursor($x,$y);
}

pub macro clear() {
    crate::terminal::_clear();
}

pub macro set_foreground($color:expr) {
    crate::terminal::_set_foreground($color);
}

pub macro set_background($color:expr) {
    crate::terminal::_set_background($color);
}

pub macro set_color($color:expr) {
    crate::terminal::_set_color($color);
}

pub macro set_cell($x:expr, $y:expr, $color:expr) {
    crate::terminal::_set_cell($x, $y, $color);
}

pub fn _get_bg_color(x:usize, y:usize) -> Color {
    WRITER.lock().buffer.get_char(x, y).color_code.get_bg_color()
}

pub fn _get_fg_color(x:usize, y:usize) -> Color {
    WRITER.lock().buffer.get_char(x, y).color_code.get_fg_color()
}

pub fn _get_char(x:usize, y:usize) -> u8 {
    WRITER.lock().buffer.get_char(x, y).ascii_char
}

pub fn _set_cell(x:usize, y:usize, color:ColorCode) {
    WRITER.lock().set_cell(x, y, color);
}

pub fn _reset_cursor() {
    WRITER.lock().reset_cursor();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::terminal::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _carriage_return() {
    WRITER.lock().carriage_return();
}

#[doc(hidden)]
pub fn _set_cursor(x:usize, y:usize) {
    WRITER.lock().set_cursor(x, y);
}

#[doc(hidden)]
pub fn _clear() {
    WRITER.lock().clear();
}

#[doc(hidden)]
pub fn _set_color(color:ColorCode) {
    WRITER.lock().set_color(color);
}

#[doc(hidden)]
pub fn _set_foreground(color:Color) {
    WRITER.lock().set_foreground(color);
}

#[doc(hidden)]
pub fn _set_background(color:Color) {
    WRITER.lock().set_background(color);
}

pub fn set_position(x:usize, y:usize) {
    WRITER.lock().row = y;
    WRITER.lock().col = x;
}

lazy_static! {
pub static ref WRITER : Mutex<TerminalWriter> = Mutex::new(
    TerminalWriter::new(ColorCode::from_u8(0x1F)));
}

use super::drivers::vga::{ColorCode, Buffer};
use super::drivers::vga::{BUFFER_HEIGHT, BUFFER_WIDTH};
pub struct TerminalWriter {
    row : usize,
    col : usize,

    color_code : ColorCode,
    buffer     : &'static mut Buffer
}

impl Printer for TerminalWriter {
    fn print_str(&mut self, s:&str) {
        for b in s.as_bytes() {
            match b {
                0x20..=0x7e | b'\n' | b'\r' | b'\t' => {
                    self.print_byte(*b);
                }

                _ => { self.print_byte(0xfe); }
            }
        }
    }

    fn carriage_return(&mut self) {
        self.col = 0;
    }
    
    fn tab(&mut self) {
        self.col += 4;
    }

    fn new_line(&mut self) {
        if self.row == BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let c = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(c);
                }
            }
            let blank = ScreenChar {
                ascii_char : b' ',
                color_code : self.color_code
            };
            for col in 0..BUFFER_WIDTH {
                self.buffer.put_char(col, self.row, blank);
            }
        } else {
            self.row += 1;
        }
        self.col = 0;
    }
}
#[allow(dead_code)]
impl TerminalWriter {
    pub fn new(color:ColorCode) -> TerminalWriter {
        TerminalWriter {
            col : 0,
            row : 0,

            color_code: color,
            buffer: get_text_mode_buffer()
        }
    }

    pub fn reset_cursor(&mut self) {
        self.row = 0;
        self.col = 0;
    }

    pub fn set_foreground(&mut self, color:Color) {
        let bg = self.color_code.as_u8() & 0xF0;
        self.color_code = ColorCode::from_u8(((bg as u8)) | color as u8);
    }

    pub fn set_background(&mut self, color:Color) {
        let fg = self.color_code.as_u8() & 0x0F;
        self.color_code = ColorCode::from_u8(((color as u8) << 4) | fg as u8);
    }

    pub fn set_cursor(&mut self, x:usize, y:usize) {
        self.buffer.set_cursor(x, y);
    }

    pub fn set_color(&mut self, color:ColorCode) {
        self.color_code = color;
    }

    pub fn clear(&mut self) {
        let blank = ScreenChar {
            ascii_char : b' ',
            color_code : self.color_code
        };
        for y in 0..BUFFER_HEIGHT {
            for x in 0..BUFFER_WIDTH {
                self.buffer.put_char(x, y, blank);
            }
        }
    }

    pub fn print_byte(&mut self, data : u8) {
        match data {
            b'\n' => self.new_line(),
            b'\r' => self.carriage_return(),
            b'\t' => self.tab(),
            data => {
                if self.col >= BUFFER_WIDTH {self.new_line()}
                let row = self.row;
                let col = self.col;

                self.buffer.put_char(col, row, ScreenChar {
                    ascii_char : data,
                    color_code : self.color_code
                });
                self.col += 1;
            }
        }
    }

    pub fn set_cell(&mut self, x:usize, y:usize, color:ColorCode) {
        self.buffer.set_cell_color(x, y, color);
    }
}

impl fmt::Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print_str(s);
        Ok(())
    }
}
