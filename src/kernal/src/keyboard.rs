use x86_64::instructions::port::Port;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyEvent};
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

pub static KEYBOARD_PORT : u16 = 0x60;
