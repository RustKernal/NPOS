# ,NPOS
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
}
```
##### Terminal Writer

```rust
//terminal.rs
use crate::vga;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref TERMINAL : Mutex<Terminal> = Mutex::new(
        Terminal::new()
    ); 
}


pub struct Terminal {
    row	   : u8,
    col	   : u8,
    color  : vga::ColorCode,
    buffer : &'static mut vga::ScreenBuffer
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
            	0x20..=0x7e => { self._print_byte(*b)    }
                _ =>			{ self._print_byte(0xFE) }
            }
        }
    }
    
    fn _print_byte(&mut self, data:u8) {
        let (max_col, _) = vga::screen_dimensions();
        if self.col >= max_col as u8 || data == b'\n' { self.new_line(); }
        if data == b'\r' { self.carriage_return(); }
        self.buffer.set_char(self.col.into(), self.row.into(), vga::Character::new(data, self.color));
        self.col += 1;
    }
    
    fn new_line(&mut self) {
        
        let (max_col, max_row) = vga::screen_dimensions();
        if self.row == (max_row - 1) as u8 {
            for y in 1..max_row {
                for x in 0..max_col {
                    let c = self.buffer.get_char(x,y);
                    self.buffer.set_char(x,y-1, c);
                }
            }
        } else {
            self.row += 1;
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

#[doc(hidden)]
pub fn _print(args : fmt::Arguments) {
    use core::fmt::Write;
    TERMINAL.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _println(args : fmt::Arguments) {
    use core::fmt::Write;
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

### Memory Allocations

#### Page Tables

The Intel 8088 could originally address **1MB** of memory using a **20-bit** Address bus.

x86_64 Expanded this to a **48-bit** Address bus, giving a maximum of **256TB**

- Bits *0 - 11* is an offset to a given page.
- Bits *12 - 21* is the **page-table** 1 index.
- Bits *21 - 30* is the **page-table** 2 index.
- Bits *30 - 39* is the **page-table** 3 index.
- Bits *39 - 48* is the **page-table** 4 index.
- the **CR3** register holds the address of the Page table 4.
- Each **Page-table** is Exactly **4096 Bytes**, and are aligned to **4KB Boundaries**
- Each **Page-table** Entry is the address of the lower **Page-table** to use, *for example, if the index at **PT** 4 is 16K, then the **page-table** at 16K is read.* 



##### Example Page-table setup

PT-4 @ 4K

| Index | PT-3 Start | FLAGS |
| ----- | ---------- | ----- |
| 0     | 8K         | RWX   |
| 1     | 12K        | RWX   |
| ...   |            |       |
| 511   | 256K       | RW    |

PT-3 @ 8K

| Index | PT-2 Start | FLAGS |
| ----- | ---------- | ----- |
| 0     | 260K       | RWX   |
| 1     | 264K       | RWX   |
| ...   |            |       |
| 511   | 512K       | RW    |

PT-2 @ 512K

| Index | PT-1 Start | FLAGS |
| ----- | ---------- | ----- |
| 0     | 516K       | RWX   |
| 1     | 520K       | RWX   |
| ...   |            |       |
| 511   | 768K       | RW    |

PT-1 @ 520K

| Index | Page Start | FLAGS |
| ----- | ---------- | ----- |
| 0     | 1024K      | RWX   |
| 1     | 1028K      | RWX   |
| ...   |            |       |
| 511   | 1032K      | RW    |

So Address 0b 000 000 001 | 111 111 111 | 111 111 111 | 111 111 111 | 0000 0000 1111 000
would map to:

PT-3 #1 -> PT-2 # 512 -> PT-1 #512 -> Offset 0x00F0.





##### Rust Implementation

```rust
//memory/paging.rs
use lazy_static::lazy_static;

pub fn translate_addr(addr : VirtAddr, physical_offset : VirtAddr) -> Option<PhysAddr> {
    _translate_addr(addr, physical_offset)
}

fn _translate_addr(addr : VirtAddr, physical_offset : VirtAddr) -> Option<PhysAddr> {
    let (pt_4_frame, _) = Cr3::read();
    let indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index()
    ];
    
    let mut current_frame = pt_4_frame;
    
    for &index in &indexes {
        let virt = physical_offset + current_frame.start_address().as_u64();
        let table_ptr : *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};
        frame = match frame {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugePage) => panic!("Huge Pages Not Supported!"),
        };
    }
    
    Some(frame.start_address() + u64::from(addr.page_offset()))
}


```





