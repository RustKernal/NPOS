use x86_64::structures::idt;
use x86_64::structures::idt::{InterruptStackFrame};
use lazy_static::lazy_static;
use spin::Mutex;


type PITHandler = idt::HandlerFunc;
pub struct Handlers {
    pit_handler : Option<PITHandler>,

    idt : idt::InterruptDescriptorTable
}


pub fn set_pit_handler(handler : PITHandler) {
    HANDLERS.lock().set_pit_handler(handler);
}

lazy_static! {
    static ref HANDLERS : Mutex<Handlers> = Mutex::new(
        Handlers {
            pit_handler : None,
            idt : idt::InterruptDescriptorTable::new()
        }
    ); 
}


impl Handlers {
    pub fn set_pit_handler(&mut self, handler : PITHandler) {
        self.pit_handler = Some(handler);
    }

}

// extern "x86-interrupt" fn breakpoint_handler(
//     stack_frame: &mut InterruptStackFrame)
// {
//     let func : Fn(_, &InterruptStackFrame) = HANDLERS.lock().pit_handler.unwrap();
// }

 