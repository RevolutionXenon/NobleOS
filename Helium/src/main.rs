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
use core::{fmt::Write, ptr::write_volatile};
#[cfg(not(test))]
use core::panic::PanicInfo;

//Constants
const HELIUM_VERSION: &str = "vDEV-2021-09-05"; //CURRENT VERSION OF KERNEL
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

    // PAGE MAP TESTING
    //Go to PML4
    let pml4 = PageMap::new(oct_to_pointer_4(PAGE_MAP_OCT, PAGE_MAP_OCT, PAGE_MAP_OCT, PAGE_MAP_OCT, 0).unwrap(), PageMapLevel::L4).unwrap();
    //Print info
    writeln!(printer, "Identity Map Area Present: {}", pml4.read_entry(PHYSICAL_OCT).unwrap().present);
    writeln!(printer, "Kernel Map Area Present:   {}", pml4.read_entry(KERNEL_OCT      ).unwrap().present);
    writeln!(printer, "Frame Buffer Area Present: {}", pml4.read_entry(FRAME_BUFFER_OCT).unwrap().present);
    writeln!(printer, "Free Memory Area Present:  {}", pml4.read_entry(FREE_MEMORY_OCT ).unwrap().present);
    writeln!(printer, "Page Map Area Present:     {}", pml4.read_entry(PAGE_MAP_OCT    ).unwrap().present);

    // PAGE MAP PARSING
    //Determine amount of free memory
    let mut identity_not_present = pml4.read_entry(PHYSICAL_OCT).unwrap();
    identity_not_present.present = false;
    pml4.write_entry(PHYSICAL_OCT, identity_not_present).unwrap();
    let pml3_free = PageMap::new(oct_to_pointer_4(PAGE_MAP_OCT, PAGE_MAP_OCT, PAGE_MAP_OCT, FREE_MEMORY_OCT, 0).unwrap(), PageMapLevel::L3).unwrap();
    let mut free_page_count: usize = 0;
    for i in 0..PAGE_NUMBER_1 {
        if pml3_free.read_entry(i).unwrap().present {
            let pml2 = PageMap::new(oct_to_pointer_4(PAGE_MAP_OCT, PAGE_MAP_OCT, FREE_MEMORY_OCT, i, 0).unwrap(), PageMapLevel::L2).unwrap();
            for j in 0..PAGE_NUMBER_1 {
                if pml2.read_entry(j).unwrap().present {
                    let pml1 = PageMap::new(oct_to_pointer_4(PAGE_MAP_OCT, FREE_MEMORY_OCT, i, j, 0).unwrap(), PageMapLevel::L1).unwrap();
                    for k in 0..PAGE_NUMBER_1 {
                        let pml1e = pml1.read_entry(k).unwrap();
                        if pml1e.present {
                            //Write bools for memory table
                            unsafe {write_volatile((oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut bool).add(free_page_count), true)}
                            free_page_count += 1;
                        }
                    }
                }
            }
        }
    }
    writeln!(printer, "Free memory found: {}Pg or {}MiB {}KiB", free_page_count, (free_page_count*PAGE_SIZE_4KIB)/MIB, ((free_page_count*PAGE_SIZE_4KIB) % MIB)/KIB);
    //"Allocate" memory space for boolean table
    for i in 0..(free_page_count+PAGE_SIZE_4KIB-1)/PAGE_SIZE_4KIB {
        unsafe {write_volatile((oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut bool).add(i), false)}
    }
    //Create free memory area allocator
    let mut free_memory_area_allocator = FreeMemoryAreaAllocator{bool_table: unsafe {core::slice::from_raw_parts_mut(oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut bool, free_page_count)}};
    let free_memory_test = free_memory_area_allocator.allocate().unwrap();
    writeln!(printer, "Free Memory Area Allocation Test: {:?}", free_memory_test);
    writeln!(printer, "Free Memory Deallocation Test:    {:?}", free_memory_area_allocator.deallocate(free_memory_test));

    // REGISTER TESTING
    let mut cr3: u64;
    let mut rip: u64;
    unsafe {
        asm!("MOV {cr3}, CR3",   cr3 = out(reg) cr3, options(nostack));
        asm!("LEA {rip}, [RIP]", rip = out(reg) rip, options(nostack));
    }
    writeln!(printer, "CR3: 0x{:16X}", cr3);
    writeln!(printer, "RIP: 0x{:16X}", rip);

    // PCI TESTING
    let mut pci_uhci_option = None;
    unsafe {for pci_bus in 0..256 {
        for pci_device in 0..32 {
            for pci_function in 0..8 {
                //Get the vendor ID here and check for if the pci device exists (vendor id == 0xFFFF means it does not)
                let pci = match Pci::new(pci_bus, pci_device, pci_function) {
                    Ok(pci) => pci,
                    Err(_) => break,
                };
                write!(printer, "PCI DEVICE:");
                write!(printer, "  Bus: {:02X}, Device: {:02X}, Function: {:01X}", pci_bus, pci_device, pci_function);
                //writeln!(printer, "  Reg0: {:08X}", pci.register(0x00).unwrap());
                //writeln!(printer, "  Reg1: {:08X}", pci.register(0x01).unwrap());
                //writeln!(printer, "  Reg2: {:08X}", pci.register(0x02).unwrap());
                //writeln!(printer, "  Reg3: {:08X}", pci.register(0x03).unwrap());
                writeln!(printer, "  |  Vendor ID: {:04X}, Device ID: {:04X}, Status: {:04X}", pci.vendor_id(), pci.device_id(), pci.status());
                //writeln!(printer, "  Revision ID:   {:02X}, Prog IF:       {:02X}, Subclass:      {:02X}, Class Code:    {:02X}", pci.revision_id(), pci.prog_if(), pci.subclass(), pci.class_code());
                //writeln!(printer, "  Cache LSZ:     {:02X}, Latency Tmr:   {:02X}, Header Type:   {:02X}, BIST:          {:02X}", pci.chache_lz(), pci.latency(), pci.header_type(), pci.bist());
                if let Ok(o) = PciUhci::new(pci) {pci_uhci_option = Some(o)};
            }
        }
    }}

    // USB TESTING
    unsafe {if let Some(mut pci_uhci) = pci_uhci_option {
        writeln!(printer, "UHCI USB Controller Found");
        for i in 0..0x0F {
            writeln!(printer, "PCI Register {:02X}: {:08X}", i, pci_uhci.pci.register(i).unwrap());
        }
        writeln!(printer, "USB Command:             {:04X}", pci_uhci.command.read());
        writeln!(printer, "USB Status:              {:04X}", pci_uhci.status.read());
        writeln!(printer, "USB Interrupt Enable:    {:04X}", pci_uhci.interrupt.read());
        writeln!(printer, "Frame Number:            {:04X}", pci_uhci.frame_num.read());
        writeln!(printer, "Frame List Base Address: {:04X}", pci_uhci.frame_base.read());
        writeln!(printer, "Frame Modify Start:      {:04X}", pci_uhci.frame_mod.read());
        writeln!(printer, "Status/Control Port 1:   {:04X}", pci_uhci.sc_1.read());
        writeln!(printer, "Status/Control Port 2:   {:04X}", pci_uhci.sc_2.read());
    }}

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


// MEMORY MANAGEMENT
struct FreeMemoryAreaAllocator<'s> {
    bool_table: &'s mut [bool]
}
impl<'s> FreeMemoryAreaAllocator<'s> {
    pub fn allocate(&mut self) -> Result<usize, &'static str> {
        for position in 0..self.bool_table.len() {
            if self.bool_table[position] {self.bool_table[position] = false; return Ok(position)}
        }
        Err("Free Memory Area Allocator: Out of memory.")
    }
    pub fn deallocate(&mut self, position: usize) -> Result<(), &'static str> {
        if self.bool_table[position] {Err("Free Memory Area Allocator: Deallocation of unallocated page.")} else {self.bool_table[position] = true; Ok(())}
    }
}
