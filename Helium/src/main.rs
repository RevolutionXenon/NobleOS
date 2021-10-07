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

//Constants
const HELIUM_VERSION: &str = "vDEV-2021-08-15"; //CURRENT VERSION OF KERNEL


// MAIN
//Main Entry Point After Hydrogen Boot
#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // GRAPHICS SETUP
    //Screen variables
    //Screen Variables
    let whitespace = Character::<BGRX_DEPTH>::new(' ', COLOR_WHT_BGRX, COLOR_BLK_BGRX);
    let redspace   = Character::<BGRX_DEPTH>::new(' ', COLOR_RED_BGRX, COLOR_BLK_BGRX);
    let renderer = Renderer::<SCREEN_H_1920_1080_BRGX, SCREEN_W_1920_1080_BRGX, BGRX_DEPTH>::new(FRAME_VIRT_PTR);
    let mut frame = CharacterFrame::<SCREEN_H_1920_1080_BRGX, SCREEN_W_1920_1080_BRGX, BGRX_DEPTH, CHARFR_H_1920_1080_BRGX, CHARFR_W_1920_1080_BRGX>::new(renderer, whitespace);
    let mut printer = PrintWindow::<SCREEN_H_1920_1080_BRGX, SCREEN_W_1920_1080_BRGX, BGRX_DEPTH, PRINTW_M_1920_1080_BRGX, PRINTW_H_1920_1080_BRGX, PRINTW_W_1920_1080_BRGX, PRINTW_Y_1920_1080_BRGX, PRINTW_X_1920_1080_BRGX>::new(renderer, whitespace, whitespace);
    let mut inputter = InputWindow::<SCREEN_H_1920_1080_BRGX, SCREEN_W_1920_1080_BRGX, BGRX_DEPTH, INPUTW_L_1920_1080_BRGX, INPUTW_W_1920_1080_BRGX, INPUTW_Y_1920_1080_BRGX, INPUTW_X_1920_1080_BRGX>::new(renderer, whitespace);
    //User Interface initialization
    frame.horizontal_line(PRINTW_Y_1920_1080_BRGX-1, 0,                         CHARFR_W_1920_1080_BRGX-1,  redspace);
    frame.horizontal_line(INPUTW_Y_1920_1080_BRGX-1, 0,                         CHARFR_W_1920_1080_BRGX-1,  redspace);
    frame.horizontal_line(INPUTW_Y_1920_1080_BRGX+1, 0,                         CHARFR_W_1920_1080_BRGX-1,  redspace);
    frame.vertical_line(  0,                         PRINTW_Y_1920_1080_BRGX-1, INPUTW_Y_1920_1080_BRGX+1,  redspace);
    frame.vertical_line(  CHARFR_W_1920_1080_BRGX-1, PRINTW_Y_1920_1080_BRGX-1, INPUTW_Y_1920_1080_BRGX+1,  redspace);
    frame.horizontal_string("NOBLE OS",      0, 0,                                                   redspace);
    frame.horizontal_string("HELIUM KERNEL", 0, CHARFR_W_1920_1080_BRGX - 14 - HELIUM_VERSION.len(), redspace);
    frame.horizontal_string(HELIUM_VERSION,  0, CHARFR_W_1920_1080_BRGX -      HELIUM_VERSION.len(), redspace);
    frame.render();
    writeln!(printer, "Welcome to Noble OS");
    writeln!(printer, "Helium Kernel           {}", HELIUM_VERSION);
    writeln!(printer, "Photon Graphics Library {}", PHOTON_VERSION);
    writeln!(printer, "Gluon Memory Library    {}", GLUON_VERSION);
    
    // HALT COMPUTER
    writeln!(printer, "Halt reached.");
    unsafe {asm!("HLT");}
    loop{}
}


// PANIC HANDLER
//Panic Handler
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    unsafe {
        let whitespace = Character::<BGRX_DEPTH>::new(' ', COLOR_WHT_BGRX, COLOR_BLK_BGRX);
            let blackspace = Character::<BGRX_DEPTH>::new(' ', COLOR_BLK_BGRX, COLOR_WHT_BGRX);
            let renderer = Renderer::<SCREEN_H_1920_1080_BRGX, SCREEN_W_1920_1080_BRGX, BGRX_DEPTH>::new(FRAME_VIRT_PTR);
            let mut printer = PrintWindow::<SCREEN_H_1920_1080_BRGX, SCREEN_W_1920_1080_BRGX, BGRX_DEPTH, PRINTW_H_1920_1080_BRGX, PRINTW_H_1920_1080_BRGX, PRINTW_W_1920_1080_BRGX, PRINTW_Y_1920_1080_BRGX, PRINTW_X_1920_1080_BRGX>::new(renderer, blackspace, whitespace);
            printer.push_render("KERNEL PANIC!\n", blackspace);
            writeln!(printer, "{}", panic_info);
        
        asm!("HLT");
        loop {}
    }
}
