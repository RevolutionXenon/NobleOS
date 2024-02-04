// NEON
// Currently a test program, to become the Noble command line shell


// HEADER
//Flags
#![no_std]
#![no_main]
#![feature(start)]


//Imports
use gluon::noble::system_calls::*;
use gluon::x86_64::instructions::hlt;
use core::panic::PanicInfo;


// MAIN
//Entry Point
#[no_mangle]
fn _start() {
    loop {
        system_call_01()
    }
}


// PANIC HANDLER
#[panic_handler]
unsafe fn panic_handler(_panic_info: &PanicInfo) -> ! {
    loop {hlt();}; //Halt the program
}
