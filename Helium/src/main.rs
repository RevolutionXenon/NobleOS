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
use core::panic::PanicInfo;
use core::ptr::write_volatile;

//Constants
const HELIUM_VERSION: &str = "v2021-08-07"; //CURRENT VERSION OF KERNEL

#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // POINTERS
    //Physical Memory
    let physm_oct_phys:      usize = 0o000;
    let physm_ptr_phys: *mut u8    = 0o000_000_000_000_0000 as *mut u8;
    let physm_oct_virt:      usize = 0o600;
    let physm_ptr_virt: *mut u8    = 0o600_000_000_000_0000 as *mut u8;
    //Kernel
    let kernl_oct_virt:      usize = 0o400;
    let kernl_ptr_virt: *mut u8    = 0o400_000_000_000_0000 as *mut u8;
    //Frame Buffer
    let frame_ptr_phys: *mut u8    = 0o000_002_000_000_0000 as *mut u8; //POSSIBLY ONLY QEMU
    let frame_oct_virt:      usize = 0o577;
    let frame_ptr_virt: *mut u8    = 0o577_000_000_000_0000 as *mut u8;
    //Page Map
    let pgmap_oct_virt:      usize = 0o777;
    let pgmap_ptr_virt: *mut u8    = 0o777_000_000_000_0000 as *mut u8;

    // OPTIONAL PANIC
    panic!();

    // DIAGNOSTIC DISPLAY
    //Write grey to screen
    unsafe{
        for i in 0..1080{
            for j in 0..1920{
                for k in 0..3{
                    write_volatile(frame_ptr_virt.add(i*1920*4 + j*4 + k), ((i*255)/1080) as u8);
                }
            }
        }
    }
    
    // HALT COMPUTER
    loop{}
}

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    let frame_ptr_phys: *mut u8    = 0o000_002_000_000_0000 as *mut u8; //POSSIBLY ONLY QEMU
    //Write red to screen
    unsafe{
        for i in 0..1080{
            for j in 0..1920{
                for k in 0..3{
                    write_volatile(frame_ptr_phys.add(i*1920*4 + j*4 + k), if k==2{0xFFu8} else {0x00u8});
                }
            }
        }
    }
    loop{}
}