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
use photon::*;
use gluon::*;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write_volatile;

//Constants
const HELIUM_VERSION: &str = "vDEV-2021-08-09"; //CURRENT VERSION OF KERNEL


// MAIN
//Main Entry Point After Hydrogen Boot
#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // GRAPHICS SETUP
    //Screen variables
    let     screen_location:  *mut u8                                               = FRAME_VIRT_PTR;
    let mut screen_charframe:      [Character; CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM]     = [Character::new(' ', COLR_PRWHT, COLR_PRBLK); CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM];
    //Input Window variables
    let mut input_stack:           [Character; CHAR_INPT_X_DIM*CHAR_INPT_Y_DIM_MEM] = [Character::new(' ', COLR_PRWHT, COLR_PRBLK); CHAR_INPT_X_DIM*CHAR_INPT_Y_DIM_MEM];
    let mut input_p:               usize                                            = 0;
    //Print Result Window variables
    let mut print_buffer:          [Character; CHAR_PRNT_X_DIM*CHAR_PRNT_Y_DIM_MEM] = [Character::new(' ', COLR_PRWHT, COLR_PRBLK); CHAR_PRNT_X_DIM*CHAR_PRNT_Y_DIM_MEM];
    let mut print_y:               usize                                            = CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP;
    let mut print_x:               usize                                            = 0;
    //Screen struct
    let mut screen:Screen = Screen{
        screen_physical:       screen_location,
        screen_charframe: &mut screen_charframe,
        input_stack:      &mut input_stack,
        input_p:          &mut input_p,
        print_buffer:     &mut print_buffer,
        print_y:          &mut print_y,
        print_x:          &mut print_x,
        print_fore:       &mut COLR_PRWHT,
        print_back:       &mut COLR_PRBLK,
    };
    //User Interface initialization
    screen.draw_hline(CHAR_PRNT_Y_POS-1, 0,                 CHAR_SCRN_X_DIM-1,  COLR_PRRED, COLR_PRBLK);
    screen.draw_hline(CHAR_INPT_Y_POS-1, 0,                 CHAR_SCRN_X_DIM-1,  COLR_PRRED, COLR_PRBLK);
    screen.draw_hline(CHAR_INPT_Y_POS+1, 0,                 CHAR_SCRN_X_DIM-1,  COLR_PRRED, COLR_PRBLK);
    screen.draw_vline(0,                 CHAR_PRNT_Y_POS-1, CHAR_INPT_Y_POS+1,  COLR_PRRED, COLR_PRBLK);
    screen.draw_vline(CHAR_SCRN_X_DIM-1, CHAR_PRNT_Y_POS-1, CHAR_INPT_Y_POS+1,  COLR_PRRED, COLR_PRBLK);
    screen.draw_string("NOBLE OS",      0, 0,                                           COLR_PRWHT, COLR_PRBLK);
    screen.draw_string("HELIUM KERNEL", 0, CHAR_SCRN_X_DIM - 14 - HELIUM_VERSION.len(), COLR_PRWHT, COLR_PRBLK);
    screen.draw_string(HELIUM_VERSION,  0, CHAR_SCRN_X_DIM -      HELIUM_VERSION.len(), COLR_PRWHT, COLR_PRBLK);
    screen.characterframe_render();
    writeln!(screen, "Welcome to Noble OS");
    writeln!(screen, "Helium Kernel           {}", HELIUM_VERSION);
    writeln!(screen, "Photon Graphics Library {}", PHOTON_VERSION);
    writeln!(screen, "Gluon Memory Library    {}", GLUON_VERSION);
    screen.printbuffer_draw_render();
    
    // HALT COMPUTER
    unsafe {asm!("HLT");}
    loop{}
}

//Panic Handler
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    //Write red to screen
    unsafe{
        for i in 0..1080{
            for j in 0..1920{
                for k in 0..3{
                    write_volatile(FRAME_VIRT_PTR.add(i*1920*4 + j*4 + k), if k==2{0x80u8} else {0x00u8});
                }
            }
        }
    }
    loop{}
}
