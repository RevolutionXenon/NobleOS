// GLUON: x86-64 INSTRUCTIONS
// Functions that shortcut intrinsic instructions from the x86-64 instruction set architecture


// HEADER
//Imports
use core::arch::asm;


// INSTRUCTIONS
//CLI: Clear Interrupt Flag
#[inline]
pub fn cli() {
    unsafe {asm!(
        "CLI",
        options(nomem, nostack)
    )}
}

//CPUID: CPU Identification
#[inline]
pub fn cpuid(leaf: u32, sub_leaf: u32) -> (u32, u32, u32, u32) {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;
    unsafe{asm!(
        "MOV {0:r}, RBX",
        "CPUID",
        "XCHG {0:r}, RBX",
        lateout(reg) ebx,
        inlateout("eax") leaf => eax,
        inlateout("ecx") sub_leaf => ecx,
        lateout("edx") edx,
        options(nostack, preserves_flags)
    )}
    (eax, ebx, ecx, edx)
}

//HLT: Halt
#[inline]
pub fn hlt() {
    unsafe {asm!(
        "HLT", 
        options(nomem, nostack, preserves_flags)
    )}
}

//IN: Input from Port (Byte)
#[inline]
pub fn in_b(port: u16) -> u8 {
    let result: u8;
    unsafe {asm!(
        "IN AL, DX",
        in("dx") port,
        out("al") result,
        options(nomem, nostack, preserves_flags)
    )}
    result
}

//IN: Input from Port (Word)
#[inline]
pub fn in_w(port: u16) -> u16 {
    let result: u16;
    unsafe {asm!(
        "IN AX, DX",
        in("dx") port,
        out("ax") result,
        options(nomem, nostack, preserves_flags)
    )}
    result
}

//IN: Input from Port (Double Word)
#[inline]
pub fn in_d(port: u16) -> u32 {
    let result: u32;
    unsafe {asm!(
        "IN EAX, DX",
        in("dx") port,
        out("eax") result,
        options(nomem, nostack, preserves_flags)
    )}
    result
}

//OUT: Output to Port (Byte)
#[inline]
pub fn out_b(port: u16, byte: u8) {
    unsafe {asm!(
        "OUT DX, AL",
        in("dx") port,
        in("al") byte,
        options(nomem, nostack, preserves_flags)
    )}
}

//OUT: Output to Port (Word)
#[inline]
pub fn out_w(port: u16, word: u16) {
    unsafe {asm!(
        "OUT DX, AX",
        in("dx") port,
        in("ax") word,
        options(nomem, nostack, preserves_flags)
    )}
}

//OUT: Output to Port (Double Word)
#[inline]
pub fn out_d(port: u16, double_word: u16) {
    unsafe {asm!(
        "OUT DX, EAX",
        in("dx") port,
        in("eax") double_word,
        options(nomem, nostack, preserves_flags)
    )}
}

//RDMSR: Read Model Specific Register
#[inline]
pub fn rdmsr(register: u32) -> u64 {
    let msr_high: u64;
    let msr_low: u64;
    unsafe {asm!(
        "RDMSR",
        in("ecx") register,
        out("rax") msr_low,
        out("rdx") msr_high,
    )}
    (msr_high << 32) + msr_low
}

//RDTSC: Read Time-Stamp Counter
#[inline]
pub fn rdtsc() -> u64 {
    let result_high: u64;
    let result_low:  u64;
    unsafe {asm!(
        "RDTSC",
        out("rax") result_low,
        out("rdx") result_high,
        options(nomem, nostack)
    )}
    (result_high << 32) + result_low
}

//STI: Set Interrupt Flag
#[inline]
pub fn sti() {
    unsafe {asm!(
        "STI",
        options(nomem, nostack)
    )}
}

//WRMSR: Write to Model Specific Register
#[inline]
pub fn wrmsr(register: u32, value: u64) {
    unsafe {asm!(
        "WRMSR",
        in("ecx") register,
        in("eax") value,
        in("edx") value >> 32
    )}
}
