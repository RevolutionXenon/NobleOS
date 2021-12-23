// GLUON: x86-64 SYSCALL
// Functions and Structs related to the handling of system calls on x86-64


// HEADER
//Imports
use x86_64::registers::model_specific::*;


// FUNCTIONS
pub unsafe fn set_handler(handler: extern "sysv64" fn(u64, u64) -> sys_return) {
    LStar::MSR.write(handler as u64);
}

pub unsafe extern "sysv64" fn systemcall(function: u64, argument: u64) -> sys_return {
    let code: u64;
    let value: u64;
    asm!(
        "SYSCALL",
        in("rdi") function,
        in("rsi") argument,
        lateout("rcx") _,
        lateout("r11") _,
        lateout("rax") code,
        lateout("rdx") value,
        clobber_abi("sysv64"),
    );
    sys_return{code, value}
}

#[naked] pub unsafe extern "sysv64" fn handler_asm() {
    asm!(
        "PUSH RCX",
        "PUSH R11",
        "CALL {handler}",
        "POP R11",
        "POP RCX",
        "SYSRET",
        handler = sym handler,
        options(noreturn),
    )
}

pub unsafe extern "sysv64" fn handler(function: u64, argument: u64) -> sys_return {
    let ret1 = function + argument;
    let ret2 = function - argument;
    sys_return{code: ret1, value: ret2}
}


// STRUCTS
#[repr(C)]
pub struct sys_return {
    pub code: u64,
    pub value: u64,
}
