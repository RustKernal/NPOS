//terminal.rs
use crate::vga;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref TERMINAL : Mutex<Terminal> = Mutex::new(
        Terminal::new()
    ); 
}


pub struct Terminal {
    pub(crate) row	   : u8,
    pub(crate) col	   : u8,
    pub(crate) color   : vga::ColorCode,
    pub(crate) buffer  : &'static mut vga::ScreenBuffer
}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {
            row     : 0,
            col     : 0,
            color   : vga::ColorCode::new(vga::Color::White, vga::Color::Blue),
            buffer  : vga::ScreenBuffer::new()
        }
    }
    
    pub fn print(&mut self, s:&str) {
        for b in s.as_bytes() {
            match b {
            	0x20..=0x7e | b'\n' => { self._print_byte(*b) }
                _ =>			{ self._print_byte(0xFE) }
            }
        }
    }
    
    fn _print_byte(&mut self, data:u8) {
        let (max_col, _) = vga::screen_dimensions();
        if self.col >= (max_col as u8)-0 || data == b'\n' { self.new_line(); return; }
        if data == b'\r' { self.carriage_return(); }
        self.buffer.set_char(self.col.into(), self.row.into(), vga::Character::new(data, self.color));
        self.col += 1;
    }
    
    fn new_line(&mut self) {
        
        let (max_col, max_row) = vga::screen_dimensions();
        if self.row == (max_row - 1) as u8 {
            for y in 1..max_row {
                for x in 0..max_col {
                    let c = self.buffer.get_char(x,y);
                    self.buffer.set_char(x,y-1, c);
                }
            }
        } else {
            self.row += 1;
        }

        self.carriage_return();
    }
    
    fn carriage_return(&mut self) {
        self.col = 0;
    }

    pub fn _clear(&mut self) {
        let c = vga::Character::new(b' ', self.color);
        let (max_col, max_row) = vga::screen_dimensions();
        for y in 0..max_row {
            for x in 0..max_col {
                self.buffer.set_char(x,y,c);
            }
        }
    }

    pub fn _set_position(&mut self, x:u8, y:u8) {
        self.col = x;
        self.row = y;
    }
}


use core::fmt;

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}


pub macro print($($arg:tt)*) {
    crate::terminal::_print(format_args!($($arg)*))
}

pub macro println($($arg:tt)*) {
    crate::terminal::print!("{}\n", format_args!($($arg)*))
}

pub macro error($($arg:tt)*) {
    {
        crate::terminal::print!("{}\n", format_args!($($arg)*))
    }
}

pub macro clear() {
    crate::terminal::_clear();
}

pub macro set_position($x:expr, $y:expr) {
    crate::terminal::_set_position($x, $y);
}

#[doc(hidden)]
pub fn _print(args : fmt::Arguments) {
    use core::fmt::Write;
    TERMINAL.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _println(args : fmt::Arguments) {
    use core::fmt::Write;
    TERMINAL.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _clear() {
    TERMINAL.lock()._clear();
}

#[doc(hidden)]
pub fn _set_position(x:usize, y:usize) {
    TERMINAL.lock()._set_position(x as u8,y as u8);
}

#[doc(hidden)]
pub fn _get_foreground(x:usize, y:usize) -> u8 {
    TERMINAL.lock().buffer.get_fg_color(x,y)
}

#[doc(hidden)]
pub fn _get_background(x:usize, y:usize) -> u8 {
    TERMINAL.lock().buffer.get_bg_color(x,y)
}