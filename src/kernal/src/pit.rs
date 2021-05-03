use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts::without_interrupts;
use crate::terminal;

pub unsafe fn set_reload_value(mut value : u16) {
    
    //terminal::println!("Setting PIT Reload Value to {}", value);

    without_interrupts( || {
        let mut data_port = Port::new(DATA_PORT_0);
        let mut command_port = Port::new(COMMAND_PORT);
        command_port.write(CHANNEL_0 | ACCESS_LOBYTE_HIBYTE | MODE_3);
        data_port.write(value & 0x00FF);
        data_port.write((value & 0xFF00) >> 8);
    }); 
}

//Runs at 1.193182MHz
pub static FREQUENCY : usize = 1_193_182;

pub static DATA_PORT_0  : u16 = 0x0040;
pub static DATA_PORT_1  : u16 = 0x0041;
pub static DATA_PORT_2  : u16 = 0x0042;
pub static COMMAND_PORT : u16 = 0x0043;


pub static CHANNEL_0 : u8 = 0b00000000;
pub static CHANNEL_1 : u8 = 0b01000000;
pub static CHANNEL_2 : u8 = 0b10000000;
pub static READ_BACK : u8 = 0b11000000;

pub static LATCH_COUNT_VALUE    : u8 = 0b00000000; 
pub static ACCESS_LOBYTE        : u8 = 0b00010000;
pub static ACCESS_HIBYTE        : u8 = 0b00100000;
pub static ACCESS_LOBYTE_HIBYTE : u8 = 0b00110000;

pub static MODE_0 : u8 = 0b0000;
pub static MODE_1 : u8 = 0b0010;
pub static MODE_2 : u8 = 0b0100;
pub static MODE_3 : u8 = 0b0110;
pub static MODE_4 : u8 = 0b1000;
pub static MODE_5 : u8 = 0b1010;
