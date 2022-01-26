// GLUON: x86-64 TIMERS
// Functions and objects related to the handling of the Local Advanced Programmable Interrupt Controller


// HEADER
//Imports
use core::{arch::x86_64::__cpuid, ptr::{read_volatile, write_volatile}};
use ::x86_64::registers::model_specific::Msr;

//Consts
const APIC_CPUID: u32 = 1<<9;
const APIC_ENABLE: u64 = 1<<11;


// LOCAL ADVANCED PROGRAMMABLE INTERRUPT CONTROLLER
static mut LAPIC_BASE_MSR: Msr = Msr::new(0x001B);
pub static mut LAPIC_ADDRESS: *mut u8 = 0xFEE00000 as *mut u8;

//CPUID Operations
pub unsafe fn apic_check() -> bool {
    let r = __cpuid(0x0001);
    r.edx & APIC_CPUID > 0
}

//Model Specific Register Operations
pub unsafe fn set_base(base: u64) -> Result<(), &'static str> {
    if base % (1<<12) != 0 {return Err("APIC Set Base: Base not aligned on 4KiB Boundary.")}
    LAPIC_BASE_MSR.write(base | APIC_ENABLE);
    Ok(())
}
pub unsafe fn get_base() -> u64 {
    LAPIC_BASE_MSR.read() & 0xFFFF_FFFF_FFFF_F000
}

//General LAPIC Register Operations
pub unsafe fn read_register(register: usize) -> Result<u32, &'static str> {
    if register % 0x10 != 0 {return Err("LAPIC Read Register: Register address not aligned.")}
    if register > 0x3F0 {return Err("LAPIC Read Register: Register out of bounds.")}
    Ok(read_volatile((LAPIC_ADDRESS.add(register)) as *mut u32))
}
pub unsafe fn write_register(register: usize, data: u32) -> Result<(), &'static str> {
    if register % 0x10 != 0 {return Err("LAPIC Write Register: Register address not aligned.")}
    if register > 0x3F0 {return Err("LAPIC Write Register: Register out of bounds.")}
    write_volatile((LAPIC_ADDRESS.add(register)) as *mut u32, data);
    Ok(())
}

//Reg 0x00B0: End of Interrupt
pub unsafe fn end_int() {
    write_register(0x00B0, 0x0000).unwrap();
}

//Reg 0x00F0: Spurious Interrupt Vector
pub unsafe fn enable() {
    write_register(0x00F0, read_register(0x00F0).unwrap() | 0x100).unwrap();
}
pub unsafe fn disable() {
    write_register(0x00F0, read_register(0x00F0).unwrap() & (!0x100)).unwrap();
}
pub unsafe fn spurious(int: u8) {
    write_register(0x00F0, read_register(0x00F0).unwrap() | int as u32).unwrap();
}

//Reg 0x0320: Local Timer
pub unsafe fn timer(vector: u8, mask: bool, mode: TimerMode) {
    write_register(0x0320, vector as u32 | (if mask {1u32} else {0u32} << 16) | ((mode as u32) << 17)).unwrap();
}
#[repr(u8)] pub enum TimerMode {
    OneShot     = 0b00,
    Periodic    = 0b01,
    TSCDeadline = 0b10,
}

//Reg 0x0380: Initial Count
pub unsafe fn initial_count(count: u32) {
    write_register(0x0380, count).unwrap();
}

//Reg 0x0390: Current Count
pub unsafe fn current_count() -> u32 {
    read_register(0x0390).unwrap()
}

//Reg 0x03E0: Divide Configuration
pub unsafe fn divide_config(div: Divide) {
    write_register(0x03E0, div as u32).unwrap();
}
#[repr(u32)] pub enum Divide {
    Divide_1   = 0b1011,
    Divide_2   = 0b0000,
    Divide_4   = 0b0001,
    Divide_8   = 0b0010,
    Divide_16  = 0b0011,
    Divide_32  = 0b1000,
    Divide_64  = 0b1001,
    Divide_128 = 0b1010,
}
