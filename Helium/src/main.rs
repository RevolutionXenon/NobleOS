// HELIUM
// Helium is the Noble Kernel:
// (PLANNED) Program loading
// (PLANNED) Thread management
// (PLANNED) Code execution
// (PLANNED) CPU time sharing
// (PLANNED) Interrupt handling
// (PLANNED) System call handling
// (PLANNED) Pipe management

// HEADER
//Flags
#![feature(start)]
#![feature(asm)]
#![no_std]
#![no_main]

//Imports
use gluon::*;
use core::panic::PanicInfo;
use core::ptr::write_volatile;

//Constants
const HELIUM_VERSION: &str = "v2021-08-08"; //CURRENT VERSION OF KERNEL

#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // DIAGNOSTIC DISPLAY
    //Write grey to screen
    unsafe{
        for i in 0..1080{
            for j in 0..1920{
                for k in 0..3{
                    write_volatile(FRAME_VIRT_PTR.add(i*1920*4 + j*4 + k), ((i*255)/1080) as u8);
                }
            }
        }
    }

    // OPTIONAL PANIC
    //panic!();
    
    // HALT COMPUTER
    loop{}
}

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    //Write red to screen
    unsafe{
        for i in 0..1080{
            for j in 0..1920{
                for k in 0..3{
                    write_volatile(FRAME_PHYS_PTR.add(i*1920*4 + j*4 + k), if k==2{0xFFu8} else {0x00u8});
                }
            }
        }
    }
    loop{}
}