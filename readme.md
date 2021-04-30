# NPOS
[TOC]
## Overview
- The Goal of this project is to bring a single tasking, terminal-based Operating System & Kernel to an x64 machine.
- The Kernel is written in Rust-lang, following the tutorial by Phillip Oppermann: https://os.phil-opp.com/
- The User-facing Side is based on Unix & Linux.
- You can find the source code here: https://github.com/RustKernal/NPOS
----
## x86-64 System Architecture
### VGA Text-mode

#### Specification

The VGA Text mode Screen buffer is located at 0xB8000, characters are stored as two-byte structures. Even Bytes store the Code point, and Odd bytes store the Attributes

##### In C

```c
struct ScreenChar {
    uint8_t codepoint,
    uint8_t bg : 4,
    uint8_t fg : 4
}
ScreenChar* screenBuffer[80 * 25] = (void*) 0xB8000;
```
The Colour Attributes are stored in this format:
| Bit 7 |    Bits 4 - 6     |    Bits 0 - 3     |
| :---: | :---------------: | :---------------: |
| Blink | Background Colour | Foreground Colour |

#### Rust Implementation
##### VGA
```rust
//drivers/vga.rs

use volatile::Volatile;

//80 x 25 Text Mode
const SCREEN_WIDTH:usize = 80; 
const SCREEN_HEIGHT:usize = 25;

pub fn screen_dimensions() -> (usize, usize) {
    return (SCREEN_WIDTH, SCREEN_HEIGHT)
}

pub enum Color {
    //.. VGA Color Codes
} 


#[repr(transparent)]
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

#[repr(C)]
pub struct Character {
    ascii_char : u8,
    color 	   : ColorCode
}

#[repr(transparent)]
pub struct ScreenBuffer {
    data : [[Volatile<Character> ; SCREEN_WIDTH] ; SCREEN_HEIGHT]
}


impl ScreenBuffer {
    
    pub fn new() -> ScreenBuffer {
        unsafe { &mut *(0xb8000 as *mut ScreenBuffer) }
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
}
```
##### Terminal Writer

```rust
//terminal.rs
use crate::drivers::vga;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static const ref TERMINAL : Mutex<Terminal> = Mutex::new(
        Terminal::new()
    ) 
};


pub struct Terminal {
    row	   : u8,
    col	   : u8,
    color  : vga::ColorCode
    buffer : vga::ScreenBuffer
}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal { buffer : vga::ScreenBuffer::new() }
    }
    
    pub fn print(&mut self, s:&str) {
        for b in s.as_bytes() {
            match b {
            	0x20 .. 0x7e => { self._print_byte(b)    }
                _ =>			{ self._print_byte(0xFE) }
            }
        }
    }
    
    fn _print_byte(&mut self, data:u8) {
        let (max_col, _) = vga::screen_dimensions();
        if self.col >= max_col | data == b'\n' { self.new_line(); }
        if data == b'\r' { self.carriage_return(); }
        self.buffer.put_char(col, row, Character {
            ascii_char:data,
            color : self.color
        });
        self.col += 1;
    }
    
    fn new_line(&mut self) {
        row += 1;
        let (max_col, max_row) = vga::screen_dimensions();
        if self.row == max_row - 1 {
            for y in 1..max_row {
                for x in 0..max_col {
                    let c = self.buffer.get_char(x,y);
                    self.buffer.set_char(x,y-1, c);
                }
            }
        }
        self.carriage_return();
    }
    
    fn carriage_return(&mut self) {
        self.col = 0;
    }
}


use core::fmt;

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}


//Requires #![feature(decl_macro)] in main.rs
pub macro print($($arg:tt)*) {
    crate::terminal::_print(format_args!($($arg)*))
}

pub macro println($($arg:tt)*) {
    crate::terminal::_println(format_args!($($arg)*))
}

pub macro println() {
    crate::terminal::_println(b'\n')
}

#[doc(hidden)]
pub fn _print(args : fmt::Arguments) {
    use core::fmt::write;
    TERMINAL.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _println(args : fmt::Arguments) {
    use core::fmt::write;
    TERMINAL.lock().write_fmt(args).unwrap();
}
```

### Interrupts / Exceptions

#### Overview

x64 Has a number of CPU Interrupts that can be thrown at any time, what happens when an exception is thrown is decided by the **Interrupt Description Table (IDT)** , a list of functions that are called when an interrupt is triggered. There is only one interrupt that we need to handle at the moment; The **Double Fault** Exception. This is thrown when an exception is thrown *inside* of another exception handler, for example if the handler doesn't exist. If the Double fault handler doesn't exist; then the CPU triggers a **Triple Fault** Exception. A Triple Fault exception ***CANNOT*** be handled, if it is thrown the system will ***RESET***.

  #### In C

```c
void doubleFaultExceptionHandler(InterruptStackFrame* frame);
```



#### Rust Implementation

##### Interrupt

```rust
//drivers/interrupts.rs

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt{InterruptDescriptionTable, InterruptStackFrame};
use crate::terminal;



lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}



pub fn init_idt() {
    IDT.load();
}

pub fn init_gdt() {
    
}

//requires #![features(x86_abi)]
extern "x86-interrupt" fn breakpoint_handler(frame : mut &InterruptStackFrame) {
    terminal::println!("{:?}", frame);
}

extern "x86-interrupt" fn double_fault_handler(frame : mut &InterruptStackFrame,
_ec : u64) -> ! {
    terminal::println!("{:?}", frame);
    loop {}
}
```

##### Global Descriptor Tables

```rust
//drivers/gdt.rs

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use lazy_static::lazy_static;
use x86_64::structures::gdt::SegmentSelector;

pub const DOUBLE_FAULT_FIRST_INDEX : u16 = 0;

pub fn init_gdt() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref TSS : TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}
```



