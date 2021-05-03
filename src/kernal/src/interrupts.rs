//drivers/interrupts.rs

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::terminal;
use crate::gdt;
use crate::pics;



lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_FIRST_INDEX);
        }

        idt[pics::InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);

        idt
    };
}



pub fn init_idt() {
    IDT.load();
}


//requires #![features(x86_abi)]
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    terminal::clear!();
    terminal::set_position!(0,0);
    terminal::println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(frame :InterruptStackFrame,
_ec : u64) -> ! {
    terminal::error!("{:?}", frame);
    loop {}
}

static mut int_count : usize = 0;

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    unsafe {int_count += 1;}
   // unsafe {terminal::println!("Hello Interrupt #{}\n", int_count)};
    pics::clear_interrupt(pics::InterruptIndex::Timer);
}