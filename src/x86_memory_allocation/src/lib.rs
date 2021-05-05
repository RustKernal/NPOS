#![no_std]
use bootloader::BootInfo;
use x86_64::{
    VirtAddr,
    PhysAddr,
    structures::paging::{PageTable, PhysFrame},
};

use x86_64::structures::paging::OffsetPageTable;

use x86_64::structures::paging::page_table::FrameError;
use x86_64::registers::control::Cr3;


pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_l4_page_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub unsafe fn to_physical_addr(address : VirtAddr, offset : VirtAddr) -> Option<PhysAddr> {
    to_physical_addr(address, offset)
}

fn _to_physical_addr(address : VirtAddr, offset : VirtAddr) -> Option<PhysAddr> {
    let (l4_frame, _) = Cr3::read();

    let table_indexes = [
        address.p4_index(), address.p3_index(), address.p2_index(), address.p1_index()
    ];

    let mut frame = l4_frame;
    for &index in &table_indexes {
        let virt = offset + frame.start_address().as_u64();
        let table_ptr : *const PageTable = virt.as_ptr();

        let mut table = unsafe {&*table_ptr};

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge Pages Not Supported!"),
        };
    }
    Some(frame.start_address() + u64::from(address.page_offset()))
}

unsafe fn active_l4_page_table(offset : VirtAddr) -> &'static mut PageTable {
    let (l4_frame, _) = Cr3::read();
    let phys = l4_frame.start_address();
    let virt = offset + phys.as_u64();
    let mut page_table_ptr : *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}  

unsafe fn active_l4_frame() -> PhysFrame {
    let (l4_frame, _) = Cr3::read();
    l4_frame
}  

