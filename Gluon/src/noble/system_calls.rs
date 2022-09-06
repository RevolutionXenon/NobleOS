// GLUON: NOBLE SYSTEM CALLS
// Structs and functions to provide system call functionality to programs running under Noble


// HEADER
//Imports
use core::arch::asm;


// STRUCTS
#[repr(C)]
struct SystemCallInternalReturnValue {
    code: u64,
    value: u64,
}


//FUNCTIONS
//Generic System Call
#[inline(always)]
extern "sysv64" fn system_call(call_number: u64, arg1: u64, arg2: u64, arg3: u64) -> SystemCallInternalReturnValue {
    let output_a: u64;
    let output_b: u64;
    unsafe {asm!(
        "INT 32h",
        in("rdi") call_number,
        in("rsi") arg1,
        in("rdx") arg2,
        in("rcx") arg3,
        lateout("rax") output_a,
        lateout("rdx") output_b,
        lateout("rdi") _,
        lateout("rsi") _,
        lateout("rcx") _,
        lateout("r8") _,
        lateout("r9") _,
        lateout("r10") _,
        lateout("r11") _,
    )}
    SystemCallInternalReturnValue {code: output_a, value: output_b}
}

//System Call 00 (Dummy Return)
#[inline(always)]
pub extern "sysv64" fn system_call_00() -> u64 {
    system_call(0, 0, 0, 0).code
}

//System Call 01 (Dummy Print)
#[inline(always)]
pub extern "sysv64" fn system_call_01() {
    system_call(1, 0, 0, 0);
}

//System Call 02 (Time)
#[inline(always)]
pub extern "sysv64" fn system_call_02() -> u64 {
    system_call(2, 0, 0, 0).code
}
