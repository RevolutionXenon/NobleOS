#![feature(start)]
#![feature(asm)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::write_volatile;

#[no_mangle]
pub extern "C" fn _start(arg: *mut u8) -> ! {
    //Write grey to screen
    unsafe{
        for i in 0..1080{
            for j in 0..1920{
                for k in 0..3{
                    write_volatile(arg.add(i*1920*4 + j*4 + k), ((i*255)/1080) as u8);
                }
            }
        }
    }
    //Halt computer
    loop{}
}

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    loop{}
}