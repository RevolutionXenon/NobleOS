// GLUON: x86-64 TIMERS
// Functions and objects related to the handling of the Programmable Interrupt Controller and Advanced Programmable Interrupt Controller


// HEADER
//Imports
use crate::*;
use core::{arch::x86_64::__cpuid, ptr::{read_volatile, write_volatile}};
use x86_64::registers::model_specific::Msr;


// PROGRAMMABLE INTERRUPT CONTROLLER
//Remap PIC to Different Interrupt Vectors
pub unsafe fn pic_remap(pic_1_offset: u8, pic_2_offset: u8) -> Result<(), &'static str> {
    if pic_1_offset % 8 != 0 || pic_2_offset % 8 != 0 {return Err("PIC: Remap offsets unaligned.")}
    //Save Masks
    let mask_1 = PORT_PIC1_DATA.read();
    let mask_2 = PORT_PIC2_DATA.read();
    //Start Initialization Sequence
    PORT_PIC1_COMMAND.write(0x11);  io_wait(); //ICW1: Start in cascade mode
    PORT_PIC2_COMMAND.write(0x11);  io_wait(); //ICW1: Start in cascade mode
    PORT_PIC1_DATA.write(pic_1_offset); io_wait(); //ICW2: Write PIC1 offset
    PORT_PIC2_DATA.write(pic_2_offset); io_wait(); //ICW2: Write PIC2 offset
    PORT_PIC1_DATA.write(0x04);     io_wait(); //ICW3: Write PIC1 PIC2 position (IRQ-2)
    PORT_PIC2_DATA.write(0x02);     io_wait(); //ICW3: Write PIC2 cascade identity
    PORT_PIC1_DATA.write(0x01);     io_wait(); //ICW4: Write PIC1 mode (8086 mode)
    PORT_PIC2_DATA.write(0x01);     io_wait(); //ICW4: Write PIC2 mode (8086 mode)
    //Rewrite Masks
    PORT_PIC1_DATA.write(mask_1);   io_wait();
    PORT_PIC2_DATA.write(mask_2);   io_wait();
    //Return
    Ok(())
}

//Set IRQ Mask
pub unsafe fn pic_set_mask(pic_1_mask: u8, pic_2_mask: u8) {
    PORT_PIC1_DATA.write(pic_1_mask);
    PORT_PIC2_DATA.write(pic_2_mask);
} 

//Enable or Disable an IRQ
pub unsafe fn pic_disable_irq(irq: u8) -> Result<(), &'static str> {
    if irq < 8 {
        PORT_PIC1_DATA.write(PORT_PIC1_DATA.read() | (1 << irq));
    }
    else if irq < 16 {
        PORT_PIC2_DATA.write(PORT_PIC2_DATA.read() | (1 << (irq-8)));
    }
    else {return Err("PIC: IRQ out of bounds on set.")}
    Ok(())
}
pub unsafe fn pic_enable_irq(irq: u8)  -> Result<(), &'static str> {
    if      irq <  8 {PORT_PIC1_DATA.write(PORT_PIC1_DATA.read() & !(1 << irq));}
    else if irq < 16 {PORT_PIC2_DATA.write(PORT_PIC2_DATA.read() & !(1 << (irq-8)));}
    else {return Err("PIC: IRQ out of bounds on clear.")}
    Ok(())
}

//Send End IRQ Signal
pub unsafe fn pic_end_irq(irq: u8) -> Result<(), &'static str> {
    if irq < 16 {
        if irq > 7 { PORT_PIC2_COMMAND.write(0x20);}
        PORT_PIC1_COMMAND.write(0x20);
        Ok(())
    }
    else {Err("PIC: IRQ out of bounds on end of interrupt.")}
}


// ADVANCED PROGRAMMABLE INTERRUPT CONTROLLER
static APIC_CPUID: u32 = 1<<9;
static APIC_ENABLE: u64 = 1<<11;
static mut LAPIC_BASE_MSR: Msr = Msr::new(0x001B);
pub static mut LAPIC_ADDRESS: *mut u8 = 0xFEE00000 as *mut u8;

//CPUID Operations
pub unsafe fn apic_check() -> bool {
    let r = __cpuid(0x0001);
    r.edx & APIC_CPUID > 0
}

//Model Specific Register Operations
pub unsafe fn lapic_set_base(base: u64) -> Result<(), &'static str> {
    if base % (1<<12) != 0 {return Err("APIC Set Base: Base not aligned on 4KiB Boundary.")}
    LAPIC_BASE_MSR.write(base | APIC_ENABLE);
    Ok(())
}
pub unsafe fn lapic_get_base() -> u64 {
    LAPIC_BASE_MSR.read() & 0xFFFF_FFFF_FFFF_F000
}

//General LAPIC Register Operations
pub unsafe fn lapic_read_register(register: usize) -> Result<u32, &'static str> {
    if register % 0x10 != 0 {return Err("LAPIC Read Register: Register address not aligned.")}
    if register > 0x3F0 {return Err("LAPIC Read Register: Register out of bounds.")}
    Ok(read_volatile((LAPIC_ADDRESS.add(register)) as *mut u32))
}
pub unsafe fn lapic_write_register(register: usize, data: u32) -> Result<(), &'static str> {
    if register % 0x10 != 0 {return Err("LAPIC Write Register: Register address not aligned.")}
    if register > 0x3F0 {return Err("LAPIC Write Register: Register out of bounds.")}
    write_volatile((LAPIC_ADDRESS.add(register)) as *mut u32, data);
    Ok(())
}

//Reg 0x00B0: End of Interrupt
pub unsafe fn lapic_end_int() {
    lapic_write_register(0x00B0, 0x0000).unwrap();
}

//Reg 0x00F0: Spurious Interrupt Vector
pub unsafe fn lapic_enable() {
    lapic_write_register(0x00F0, lapic_read_register(0x00F0).unwrap() | 0x100).unwrap();
}
pub unsafe fn lapic_disable() {
    lapic_write_register(0x00F0, lapic_read_register(0x00F0).unwrap() & (!0x100)).unwrap();
}
pub unsafe fn lapic_spurious(int: u8) {
    lapic_write_register(0x00F0, lapic_read_register(0x00F0).unwrap() | int as u32).unwrap();
}

//Reg 0x0320: Local Timer
pub unsafe fn lapic_timer(vector: u8, mask: bool, mode: TimerMode) {
    lapic_write_register(0x0320, vector as u32 | (if mask {1u32} else {0u32} << 16) | ((mode as u32) << 17)).unwrap();
}
#[repr(u8)] pub enum TimerMode {
    OneShot     = 0b00,
    Periodic    = 0b01,
    TSCDeadline = 0b10,
}

//Reg 0x0380: Initial Count
pub unsafe fn lapic_initial_count(count: u32) {
    lapic_write_register(0x0380, count).unwrap();
}

//Reg 0x0390: Current Count
pub unsafe fn lapic_current_count() -> u32 {
    lapic_read_register(0x0390).unwrap()
}

//Reg 0x03E0: Divide Configuration
pub unsafe fn lapic_divide_config(div: LapicDivide) {
    lapic_write_register(0x03E0, div as u32).unwrap();
}
#[repr(u32)] pub enum LapicDivide {
    Divide_1   = 0b1011,
    Divide_2   = 0b0000,
    Divide_4   = 0b0001,
    Divide_8   = 0b0010,
    Divide_16  = 0b0011,
    Divide_32  = 0b1000,
    Divide_64  = 0b1001,
    Divide_128 = 0b1010,
}
