// GLUON: x86-64 INSTRUCTIONS


// HEADER
//Imports
use core::arch::asm;


// INSTRUCTIONS
//CLI: Clear Interrupt Flag
#[inline(always)] pub fn cli() {unsafe {asm!("CLI", options(nomem, nostack));}}

//HLT: Halt
#[inline(always)] pub fn hlt() {unsafe {asm!("HLT", options(nomem, nostack, preserves_flags));}}

//RDMSR: Read Model Specific Register
#[inline]
pub fn rdmsr(register: u32) -> u64 {
    let msr_high: u64;
    let msr_low: u64;
    unsafe {asm!("RDMSR", in("ecx") register, out("rax") msr_low, out("rdx") msr_high);}
    (msr_high << 32) + msr_low
}

//RDTSC: Read Time-Stamp Counter
#[inline]
pub fn rdtsc() -> u64 {
    let result_high: u64;
    let result_low:  u64;
    unsafe {asm!("RDTSC", out("rax") result_low, out("rdx") result_high, options(nomem, nostack));}
    (result_high << 32) + result_low
}

//STI: Set Interrupt Flag
#[inline(always)] pub fn sti() {unsafe {asm!("STI", options(nomem, nostack));}}

//WRMSR: Write to Model Specific Register
#[inline]
pub fn wrmsr(register: u32, value: u64) {
    unsafe {asm!("WRMSR", in("ecx") register, in("eax") value, in("edx") value >> 32)}
}
