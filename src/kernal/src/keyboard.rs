use x86_64::instructions::port::Port;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyEvent, KeyCode};
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
            HandleControl::Ignore)
        );
}

pub fn read_scancode() -> u8 {
    unsafe {
        let mut port = Port::new(KEYBOARD_PORT);
        return port.read();
    }
}

pub fn read_key() -> Option<DecodedKey> {
    let scancode = read_scancode();
    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            return Some(key);
        }
    }
    return None;
}

pub fn read_unicode_key() -> Option<char> {
    if let Some(key) = read_key() {
        return match key {
            DecodedKey::Unicode(chr) => {Some(chr)},
            _ => {None}
        }
    } else {
        return None;
    } 
}

pub fn read_rawkey() -> Option<KeyCode> {
    if let Some(key) = read_key() {
        return match key {
            DecodedKey::RawKey(chr) => {Some(chr)},
            _ => {None}
        }
    } else {
        return None;
    } 
}

pub fn set_leds_state(state : u8) {
    unsafe {
        let mut port = Port::new(KEYBOARD_PORT);
        port.write(LED_STATE);
        port.read();
        port.write(state);
        port.read();
    }
}

pub static KEYBOARD_PORT : u16 = 0x60;
pub static LED_STATE : u8 = 0xED;
pub static COMMAND_ACK : u8 = 0xFA;

pub static LED_SCROLL_ON : u8 = 0b001;
pub static LED_SCROLL_OFF : u8 = 0b000;

pub static LED_NUM_LOCK_ON : u8 = 0b010;
pub static LED_NUM_LOCK_OFF : u8 = 0b000;

pub static LED_CAPS_LOCK_ON : u8 = 0b100;
pub static LED_CAPS_LOCK_OFF : u8 = 0b000;