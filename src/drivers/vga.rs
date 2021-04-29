

use volatile::Volatile;
pub const BUFFER_HEIGHT : usize = 25;
pub const BUFFER_WIDTH  : usize = 80;

pub fn get_text_mode_buffer() -> &'static mut Buffer {
    unsafe { &mut *(0xb8000 as *mut Buffer) } 
}



#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl Color {
    pub fn from_u8(data : u8) -> Color {
        match data {
            0  => Color::Black,
            1  => Color::Blue,
            2  => Color::Green,
            3  => Color::Cyan,
            4  => Color::Red,
            5  => Color::Magenta,
            6  => Color::Brown,
            7  => Color::LightGray,
            8  => Color::DarkGray,
            9  => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::Pink,
            14 => Color::Yellow,
            15 => Color::White,
            _  => Color::White
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(fg : Color, bg : Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | fg as u8)
    }

    pub fn from_u8(value : u8) -> ColorCode {
        ColorCode(value)
    }

    pub fn as_u8(&mut self) -> u8 {
        self.0
    }

    pub fn get_bg_color(&mut self) -> Color {
        Color::from_u8((self.as_u8() & 0xF0) >> 4)
    }

    pub fn get_fg_color(&mut self) -> Color {
        Color::from_u8(self.as_u8() & 0x0F)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_char : u8,
    pub color_code : ColorCode
}

#[repr(transparent)]
pub struct Buffer {
    pub chars : [[Volatile<ScreenChar> ; BUFFER_WIDTH] ; BUFFER_HEIGHT]
}

#[allow(dead_code)]
impl Buffer {

    pub fn put_char(&mut self, x:usize, y:usize, c:ScreenChar) {
        self.chars[y][x].write(c);
    }

    pub fn get_char(&mut self, x:usize, y:usize) -> ScreenChar {
        self.chars[y][x].read()
    }

    pub fn set_cursor(&mut self, x:usize, y:usize) {
        let data : u8 = self.get_char(x, y).ascii_char;
        self.put_char(x, y, ScreenChar {
            ascii_char : data,
            color_code : ColorCode::from_u8(0xAA)
        });
    }

    pub fn set_cell_color(&mut self, x:usize, y:usize, color:ColorCode) {
        self.put_char(x, y, ScreenChar {ascii_char: b' ', color_code:color});
    }
}


