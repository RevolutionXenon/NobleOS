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
#![no_std]
#![no_main]
#![allow(unused_must_use)]
#![feature(start)]
#![feature(asm)]

//Imports
use photon::*;
use gluon::*;
use core::fmt::Write;
#[cfg(not(test))]
use core::panic::PanicInfo;

//Constants
const HELIUM_VERSION: &str = "vDEV-2021-08-24"; //CURRENT VERSION OF KERNEL


// MAIN
//Main Entry Point After Hydrogen Boot
#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // GRAPHICS SETUP
    //Screen variables
    //Screen Variables
    let whitespace = Character::<BGRX_DEPTH>::new(' ', COLOR_WHT_BGRX, COLOR_BLK_BGRX);
    let redspace   = Character::<BGRX_DEPTH>::new(' ', COLOR_RED_BGRX, COLOR_BLK_BGRX);
    let renderer = Renderer::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH>::new(FRAME_BUFFER_VIRTUAL_POINTER);
    let mut frame = CharacterFrame::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH, F1_FRAME_HEIGHT, F1_FRAME_WIDTH>::new(renderer, whitespace);
    let mut printer = PrintWindow::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH, F1_PRINT_LINES, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, F1_PRINT_Y, F1_PRINT_X>::new(renderer, whitespace, whitespace);
    let mut inputter = InputWindow::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH, F1_INPUT_LENGTH, F1_INPUT_WIDTH, F1_INPUT_Y, F1_INPUT_X>::new(renderer, whitespace);
    //User Interface initialization
    frame.horizontal_line(F1_PRINT_Y-1, 0,                         F1_FRAME_WIDTH-1,  redspace);
    frame.horizontal_line(F1_INPUT_Y-1, 0,                         F1_FRAME_WIDTH-1,  redspace);
    frame.horizontal_line(F1_INPUT_Y+1, 0,                         F1_FRAME_WIDTH-1,  redspace);
    frame.vertical_line(  0,                         F1_PRINT_Y-1, F1_INPUT_Y+1,  redspace);
    frame.vertical_line(  F1_FRAME_WIDTH-1, F1_PRINT_Y-1, F1_INPUT_Y+1,  redspace);
    frame.horizontal_string("NOBLE OS",      0, 0,                                                   redspace);
    frame.horizontal_string("HELIUM KERNEL", 0, F1_FRAME_WIDTH - 14 - HELIUM_VERSION.len(), redspace);
    frame.horizontal_string(HELIUM_VERSION,  0, F1_FRAME_WIDTH -      HELIUM_VERSION.len(), redspace);
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
#[cfg(not(test))]
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    unsafe {
        let whitespace = Character::<BGRX_DEPTH>::new(' ', COLOR_WHT_BGRX, COLOR_BLK_BGRX);
            let blackspace = Character::<BGRX_DEPTH>::new(' ', COLOR_BLK_BGRX, COLOR_WHT_BGRX);
            let renderer = Renderer::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH>::new(FRAME_BUFFER_VIRTUAL_POINTER);
            let mut printer = PrintWindow::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH, F1_PRINT_HEIGHT, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, F1_PRINT_Y, F1_PRINT_X>::new(renderer, blackspace, whitespace);
            printer.push_render("KERNEL PANIC!\n", blackspace);
            writeln!(printer, "{}", panic_info);
        
        asm!("HLT");
        loop {}
    }
}
