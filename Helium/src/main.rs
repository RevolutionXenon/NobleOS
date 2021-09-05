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
#![feature(asm)]
#![feature(start)]

//Imports
use photon::*;
use gluon::*;
use core::{fmt::Write};
#[cfg(not(test))]
use core::panic::PanicInfo;

//Constants
const HELIUM_VERSION: &str = "vDEV-2021-09-04"; //CURRENT VERSION OF KERNEL
static WHITESPACE:  CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK};
static _BLACKSPACE: CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLACK, background: COLOR_BGRX_WHITE};
static _BLUESPACE:  CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLUE,  background: COLOR_BGRX_BLACK};
static REDSPACE:    CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_RED, background: COLOR_BGRX_BLACK};


// MAIN
//Main Entry Point After Hydrogen Boot
#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // GRAPHICS SETUP
    //Screen Variables
    let pixel_renderer: PixelRendererHWD<ColorBGRX> = PixelRendererHWD {pointer: oct4_to_pointer(FRAME_BUFFER_OCT).unwrap() as *mut ColorBGRX, height: F1_SCREEN_HEIGHT, width: F1_SCREEN_WIDTH};
    let character_renderer: CharacterTwoToneRenderer16x16<ColorBGRX> = CharacterTwoToneRenderer16x16::<ColorBGRX> {renderer: &pixel_renderer, height: F1_FRAME_HEIGHT, width: F1_FRAME_WIDTH, y: 0, x: 0};
    let mut frame: FrameWindow::<F1_FRAME_HEIGHT, F1_FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = FrameWindow::<F1_FRAME_HEIGHT, F1_FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, 0, 0);
    let mut _inputter: InputWindow::<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = InputWindow::<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, F1_INPUT_Y, F1_INPUT_X);
    let mut printer: PrintWindow::<F1_PRINT_LINES, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = PrintWindow::<F1_PRINT_LINES, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, WHITESPACE, F1_PRINT_Y, F1_PRINT_X);
    unsafe {PANIC_WRITE_POINTER = Some(&mut printer as &mut dyn Write as *mut dyn Write)};
    //User Interface initialization
    frame.horizontal_line(F1_PRINT_Y-1, 0, F1_FRAME_WIDTH-1, REDSPACE);
    frame.horizontal_line(F1_INPUT_Y-1, 0, F1_FRAME_WIDTH-1, REDSPACE);
    frame.horizontal_line(F1_INPUT_Y+1, 0, F1_FRAME_WIDTH-1, REDSPACE);
    frame.vertical_line(0, F1_PRINT_Y-1, F1_INPUT_Y+1, REDSPACE);
    frame.vertical_line(F1_FRAME_WIDTH-1, F1_PRINT_Y-1, F1_INPUT_Y+1, REDSPACE);
    frame.horizontal_string("NOBLE OS", 0, 0, REDSPACE);
    frame.horizontal_string("HELIUM KERNEL", 0, F1_FRAME_WIDTH - 14 - HELIUM_VERSION.len(), REDSPACE);
    frame.horizontal_string(HELIUM_VERSION, 0, F1_FRAME_WIDTH - HELIUM_VERSION.len(), REDSPACE);
    frame.render();
    writeln!(printer, "Welcome to Noble OS");
    writeln!(printer, "Helium Kernel           {}", HELIUM_VERSION);
    writeln!(printer, "Photon Graphics Library {}", PHOTON_VERSION);
    writeln!(printer, "Gluon Memory Library    {}", GLUON_VERSION);

    // TEST WRITES
    for i in 0..100 {
        writeln!(printer, "Test Write: {}", i);
    }

    // PAGE MAP PARSING
    //Go to PML4
    let pml4 = PageMap::new(oct_to_pointer_4(PAGE_MAP_OCT, PAGE_MAP_OCT, PAGE_MAP_OCT, PAGE_MAP_OCT, 0).unwrap(), PageMapLevel::L4).unwrap();
    //Print info
    writeln!(printer, "Identity Map Area Present: {}", pml4.read_entry(IDENTITY_MAP_OCT).unwrap().present);
    writeln!(printer, "Kernel Map Area Present:   {}", pml4.read_entry(KERNEL_OCT      ).unwrap().present);
    writeln!(printer, "Frame Buffer Area Present: {}", pml4.read_entry(FRAME_BUFFER_OCT).unwrap().present);
    writeln!(printer, "Free Memory Area Present:  {}", pml4.read_entry(FREE_MEMORY_OCT ).unwrap().present);
    writeln!(printer, "Page Map Area Present:     {}", pml4.read_entry(PAGE_MAP_OCT    ).unwrap().present);
    //Determine amount of free memory
    let pml3_free = PageMap::new(oct_to_pointer_4(PAGE_MAP_OCT, PAGE_MAP_OCT, PAGE_MAP_OCT, FREE_MEMORY_OCT, 0).unwrap(), PageMapLevel::L3).unwrap();
    let mut free_page_count: usize = 0;
    for i in 0..PAGE_NUMBER_1 {
        if pml3_free.read_entry(i).unwrap().present {
            let pml2 = PageMap::new(oct_to_pointer_4(PAGE_MAP_OCT, PAGE_MAP_OCT, FREE_MEMORY_OCT, i, 0).unwrap(), PageMapLevel::L2).unwrap();
            for j in 0..PAGE_NUMBER_1 {
                if pml2.read_entry(j).unwrap().present {
                    let pml1 = PageMap::new(oct_to_pointer_4(PAGE_MAP_OCT, FREE_MEMORY_OCT, i, j, 0).unwrap(), PageMapLevel::L1).unwrap();
                    for k in 0..PAGE_NUMBER_1 {
                        if pml1.read_entry(k).unwrap().present {
                            free_page_count += 1;
                        }
                    }
                }
            }
        }
    }
    writeln!(printer, "Free memory found: {}Pg or {}MiB {}KiB", free_page_count, (free_page_count*PAGE_SIZE_4KIB)/MIB, ((free_page_count*PAGE_SIZE_4KIB) % MIB)/KIB);

    

    // HALT COMPUTER
    writeln!(printer, "Halt reached.");
    unsafe {asm!("HLT");}
    panic!("Kernel exited.")
}


// PANIC HANDLER
//Panic Variables
static mut PANIC_WRITE_POINTER: Option<*mut dyn Write> = None;

//Panic Handler
#[cfg(not(test))]
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    unsafe {
        let printer = &mut *PANIC_WRITE_POINTER.unwrap();
        writeln!(printer, "{}", panic_info);
        asm!("HLT");
        loop {}
    }
}
