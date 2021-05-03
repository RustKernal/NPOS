use volatile::Volatile;

//80 x 25 Text Mode
const SCREEN_WIDTH:usize = 80; 
const SCREEN_HEIGHT:usize = 25;
const TEXT_MODE_START:usize = 0xb8000;

pub fn screen_dimensions() -> (usize, usize) {
    return (SCREEN_WIDTH, SCREEN_HEIGHT)
}

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


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(fg:Color, bg:Color) -> ColorCode {
        ColorCode ((bg as u8) << 4 | fg as u8)
    }
    
    pub fn as_u8(&mut self) -> u8 {
        self.0
    }
    
    pub fn get_background(&mut self) -> u8 {
        (self.as_u8() >> 4)
    }
    
    pub fn get_foreground(&mut self) -> u8 {
        (self.as_u8() & 0x0F)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Character {
    ascii_char : u8,
 
    color 	   : ColorCode
}

impl Character {
    pub fn new(ascii_char : u8, color : ColorCode) -> Character {
        Character {ascii_char : ascii_char, color : color}
    }
}

#[repr(transparent)]
pub struct ScreenBuffer {
    data : [[Volatile<Character> ; SCREEN_WIDTH] ; SCREEN_HEIGHT]
}


impl ScreenBuffer {
    
    pub fn new() -> &'static mut ScreenBuffer {
        unsafe { &mut *(TEXT_MODE_START as *mut ScreenBuffer) }
    }
    
	pub fn get_codepoint(&mut self, x:usize, y:usize) -> u8 {
        self.get_char(x,y).ascii_char
    }
    
    pub fn get_char(&mut self, x:usize, y:usize) -> Character {
        self.data[y][x].read()
    }
    
    pub fn set_char(&mut self, x:usize, y:usize, chr:Character) {
        self.data[y][x].write(chr);
    }
    
    pub fn get_fg_color(&mut self, x:usize, y:usize) -> u8 {
        self.get_char(x,y).color.get_foreground()
    }
    
    pub fn get_bg_color(&mut self, x:usize, y:usize) -> u8 {
        self.get_char(x,y).color.get_background()
    }

    fn check_bound(x:usize, y:usize) {
        if (
            (x < 0 || x >= SCREEN_WIDTH) &&
            (y < 0 || y >= SCREEN_HEIGHT)
        ) {
            panic!("Bounds At [{},{}] is out of range for the screen buffer", x, y);
        }
    }       
}