//drivers/interrupts.rs

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::terminal;
use crate::gdt;
use crate::pics;
use crate::keyboard;
use crate::serial;

use pc_keyboard::KeyCode;



lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_FIRST_INDEX);
        }

        idt[pics::InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[pics::InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler); 


        idt
    };
}



pub fn init_idt() {
    serial::print!("Loading IDT...");
    IDT.load();
    serial::println!("[OK]");
}


//requires #![features(x86_abi)]
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame)
{
    terminal::clear!();
    terminal::set_position!(0,0);
    terminal::println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(frame : &mut InterruptStackFrame,
_ec : u64) -> ! {
    terminal::error!("{:?}", frame);
    loop {}
}

static mut int_count : usize = 0;

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    unsafe {int_count += 1;}
   // unsafe {terminal::println!("Hello Interrupt #{}\n", int_count)};
    pics::clear_interrupt(pics::InterruptIndex::Timer);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame:  &mut InterruptStackFrame)
{
    if let Some(key) = keyboard::read_unicode_key() {
        
        if key == '\u{8}' {
            terminal::backspace()
        } else {
            if terminal::get_column() < 79 {
                terminal::print!("{:}",key);
            } else {
                terminal::newline();
            }
        }
    } else {
        if let Some(key) = keyboard::read_rawkey() {
            match key {
                KeyCode::F1 => {terminal::clear!(); terminal::set_position!(0,0); terminal::update_cursor();}
                _ => {}
            }
        }
    }

    unsafe {
        pics::PICS.lock()
            .notify_end_of_interrupt(pics::InterruptIndex::Keyboard.as_u8());
    }
}