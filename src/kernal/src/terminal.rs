//terminal.rs
use crate::vga;
use lazy_static::lazy_static;
use spin::Mutex;

use x86_64::instructions::interrupts::without_interrupts;

pub static TAB_LENGTH : usize = 4;

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
            	0x20..=0x7e | b'\n' | b'\t' => { self._print_byte(*b) }
                _ =>			{ self._print_byte(0xFE) }
            }
        }
    }
    
    fn _print_byte(&mut self, data:u8) {
        if self.col >= 79 {self.new_line()}
        let (max_col, _) = vga::screen_dimensions();
        if self.col == (max_col as u8) || data == b'\n' { self.new_line(); return; }
        if data == b'\r' { self.carriage_return(); return; }
        if data == b'\t' { self.tab(); return; }
        self.buffer.set_char(self.col.into(), self.row.into(), vga::Character::new(data, self.color));
        self.col += 1;

        self.cursor(self.col.into(), self.row.into());
    }
    
    fn new_line(&mut self) {
        self.clear_cursor(self.col as usize, self.row as usize );
        let (max_col, max_row) = vga::screen_dimensions();
        if self.row == (max_row - 1) as u8 {
            for y in 1..max_row {
                for x in 0..max_col {
                    let c = self.buffer.get_char(x,y);
                    self.buffer.set_char(x,y-1, c);
                }
            }
            self._clear_row();
        } else {
            self.row += 1;
        }

        self.carriage_return();
        self.cursor(self.col.into(), self.row.into());
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

    pub fn _clear_row(&mut self) {
        let c = vga::Character::new(b' ', self.color);
        let (max_col, max_row) = vga::screen_dimensions();
            for x in 0..max_col {
                self.buffer.set_char(x,self.row.into(),c);
            }
    }

    pub fn _set_position(&mut self, x:u8, y:u8) {
        self.col = x;
        self.row = y;
    }

    pub fn set_bg_color(&mut self, color : u8) {
        self.color.set_background(color);
    }

    pub fn set_color_u8(&mut self, fg : u8, bg : u8) {
        self.color = vga::ColorCode::from_u8s(fg, bg);
    }

    pub fn tab(&mut self) {
        //if self.col >= 75 {self.new_line()}
        for _ in 0..TAB_LENGTH {
            self._print_byte(b' ');
        }
    }

    pub fn backspace(&mut self) {
        self.clear_cursor(self.col as usize, self.row as usize );
        if self.col == 0 {
            if self.row > 0 {
                self.row -= 1;
                self.col = 79;
            }
        } 
        if self.col > 0 {
            self.col -= 1;
            self._print_byte(b' ');
            self.col -= 1;
        }
        self.clear_cursor(self.col as usize + 1, self.row as usize);
        self.cursor(self.col as usize, self.row as usize );
    } 


    pub fn cursor(&mut self, x:usize, y:usize) {
        self.buffer.set_cell_attribs(x,y, vga::ColorCode::new(
            vga::Color::Black,
            vga::Color::White,
        ));
    }

    pub fn clear_cursor(&mut self, x:usize, y:usize) {
        self.buffer.set_cell_attribs(x,y, vga::ColorCode::new(
            vga::Color::White,
            vga::Color::Blue,
        ));
    }

    pub fn translate_cursor(&mut self, x:isize, y:isize) {
        let (max_col, max_row) = vga::screen_dimensions();
        self.clear_cursor(self.col as usize, self.row as usize);
        if 
            self.col > 0 && self.col < max_col as u8 &&
            self.row > 0 && self.row < max_row as u8
        {
            self.col += x as u8;
            self.row += y as u8;
        }
        self.cursor(self.col as usize, self.row as usize );
    }

    pub fn update_cursor(&mut self) {
        self.clear_cursor(self.col as usize, self.row as usize);
        self.cursor(self.col as usize, self.row as usize );
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

        crate::terminal::print!("{}\n", format_args!($($arg)*))
}

pub macro clear() {
    crate::terminal::_clear();
}

pub macro set_position($x:expr, $y:expr) {
    crate::terminal::_set_position($x, $y);
}

pub macro set_background($color:expr) {
    crate::terminal::_set_bg_color($color);
}

#[doc(hidden)]
pub fn _print(args : fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(|| { 
        TERMINAL.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _println(args : fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(|| { 
        TERMINAL.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _clear() {
    without_interrupts(|| { 
        TERMINAL.lock()._clear();
    });
}

#[doc(hidden)]
pub fn _set_position(x:usize, y:usize) {
    without_interrupts(|| { 
        TERMINAL.lock()._set_position(x as u8,y as u8);
    });
}

#[doc(hidden)]
pub fn _set_bg_color(color : u8) {
    without_interrupts(|| { 
        TERMINAL.lock().set_bg_color(color);
    });
}

#[doc(hidden)]
pub fn _get_foreground(x:usize, y:usize) -> u8 {
    let mut c : u8 = 0;
    without_interrupts(|| { 
        c = TERMINAL.lock().buffer.get_fg_color(x,y)
    });
    c
}

#[doc(hidden)]
pub fn _get_background(x:usize, y:usize) -> u8 {
    let mut c : u8 = 0;
    without_interrupts(|| { 
        c = TERMINAL.lock().buffer.get_bg_color(x,y)
    });
    c
}

pub fn set_color_u8(fg : u8, bg : u8) {
    without_interrupts(|| {
        TERMINAL.lock().set_color_u8(fg, bg);
    });
}

pub fn clear_row() {
    without_interrupts(|| {
        TERMINAL.lock()._clear_row();
    });
}

pub fn newline() {
    without_interrupts(|| {
        TERMINAL.lock().new_line();
    });
}

pub fn backspace() {
    without_interrupts(|| {
        TERMINAL.lock().backspace();
    });
}

pub fn tab() {
    without_interrupts(|| {
        TERMINAL.lock().tab();
    });
}

pub fn update_cursor() {
    without_interrupts(|| {
        TERMINAL.lock().update_cursor();
    });
}

pub fn cursor(x:usize, y:usize) {
    without_interrupts(|| {
        TERMINAL.lock().cursor(x,y);
    });
}

pub fn translate_cursor(x:isize, y:isize) {
    without_interrupts(|| {
        TERMINAL.lock().translate_cursor(x,y);
    });
}

pub fn get_column() -> u8 {
    let mut c : u8 = 0;
    without_interrupts(|| {
        c = TERMINAL.lock().col;
    });
    c
}