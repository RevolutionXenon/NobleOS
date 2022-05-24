
// HEADER
//Imports
use core::arch::asm;
use super::paging::PhysicalAddress;


//FUNCTIONS
pub fn read_cr2() -> u64 {
    let value: u64;
    unsafe{asm!("MOV {}, CR2", out(reg) value, options(nomem, nostack, preserves_flags));}
    value
}

pub fn read_cr3() -> u64 {
    let value: u64;
    unsafe{asm!("MOV {}, CR3", out(reg) value, options(nomem, nostack, preserves_flags));}
    value
}

pub fn read_cr3_address() -> PhysicalAddress {
    let value: u64;
    unsafe{asm!("MOV {}, CR3", out(reg) value, options(nomem, nostack, preserves_flags));}
    PhysicalAddress((value & 0xFFFF_FFFF_FFFF_F000) as usize)
}
