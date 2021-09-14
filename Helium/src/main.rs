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
#![feature(panic_info_message)]
#![feature(start)]

//Imports
use photon::*;
use gluon::*;
use gluon::mem::*;
use gluon::pci::*;
use gluon::ps2::*;
use x86_64::registers::control::Cr3;
use core::{fmt::Write, ptr::{read_volatile, write_volatile}, slice::from_raw_parts_mut};
#[cfg(not(test))]
use core::panic::PanicInfo;

//Constants
const HELIUM_VERSION: &str = "vDEV-2021-09-14"; //CURRENT VERSION OF KERNEL
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

    // REGISTER TESTING
    writeln!(printer, "\n=== REGISTER TEST ===\n");
    let mut cr3: u64;
    let mut rip: u64;
    unsafe {
        asm!("MOV {cr3}, CR3",   cr3 = out(reg) cr3, options(nostack));
        asm!("LEA {rip}, [RIP]", rip = out(reg) rip, options(nostack));
    }
    writeln!(printer, "CR3: 0x{:16X}", cr3);
    writeln!(printer, "RIP: 0x{:16X}", rip);

    // PAGE MAP PARSING
    writeln!(printer, "\n=== PAGE MAP ===\n");
    //Create "allocator" for address translation
    let none_alloc = NoneAllocator{identity_offset: oct4_to_usize(IDENTITY_OCT).unwrap()};
    //Go to PML4
    let pml4_physical = PhysicalAddress(Cr3::read().0.start_address().as_u64() as usize);
    let pml4 = PageMap::new(pml4_physical, PageMapLevel::L4, &none_alloc).unwrap();
    //Print info
    writeln!(printer, "Physical Memory Area Present: {}", pml4.read_entry(PHYSICAL_OCT    ).unwrap().present);
    writeln!(printer, "Kernel Area Present:          {}", pml4.read_entry(KERNEL_OCT      ).unwrap().present);
    writeln!(printer, "Programs Area Present:        {}", pml4.read_entry(PROGRAMS_OCT    ).unwrap().present);
    writeln!(printer, "Frame Buffer Area Present:    {}", pml4.read_entry(FRAME_BUFFER_OCT).unwrap().present);
    writeln!(printer, "Free Memory Area Present:     {}", pml4.read_entry(FREE_MEMORY_OCT ).unwrap().present);
    writeln!(printer, "Offset Identity Area Present: {}", pml4.read_entry(IDENTITY_OCT    ).unwrap().present);
    writeln!(printer, "Page Map Area Present:        {}", pml4.read_entry(PAGE_MAP_OCT    ).unwrap().present);
    //Remove physical memory area
    {
        let mut identity_not_present = pml4.read_entry(PHYSICAL_OCT).unwrap();
        identity_not_present.present = false;
        pml4.write_entry(PHYSICAL_OCT, identity_not_present).unwrap();
    }
    //Determine amount of free memory
    let mut free_page_count: usize = 0;
    {
        let pml3_free = PageMap::new(pml4.read_entry(FREE_MEMORY_OCT).unwrap().physical, PageMapLevel::L3, &none_alloc).unwrap();
        for i in 0..PAGE_NUMBER_1 {
            let pml3_free_entry = pml3_free.read_entry(i).unwrap();
            if pml3_free_entry.present {
                let pml2 = PageMap::new(pml3_free_entry.physical, PageMapLevel::L2, &none_alloc).unwrap();
                for j in 0..PAGE_NUMBER_1 {
                    let pml2_entry = pml2.read_entry(j).unwrap();
                    if pml2_entry.present {
                        let pml1 = PageMap::new(pml2_entry.physical, PageMapLevel::L1, &none_alloc).unwrap();
                        for k in 0..PAGE_NUMBER_1 {
                            let pml1e = pml1.read_entry(k).unwrap();
                            if pml1e.present {
                                //Write bools for memory table
                                unsafe {write_volatile((oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut UsableMemoryEntry).add(free_page_count), UsableMemoryEntry {address: pml1e.physical, present: true})}
                                free_page_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    writeln!(printer, "Free memory found: {}Pg or {}MiB {}KiB", free_page_count, (free_page_count*PAGE_SIZE_4KIB)/MIB, ((free_page_count*PAGE_SIZE_4KIB) % MIB)/KIB);
    //"Allocate" memory space for table
    let usable_table = unsafe {from_raw_parts_mut(oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut UsableMemoryEntry, free_page_count)};
    for i in 0..(free_page_count+PAGE_SIZE_4KIB-1)/PAGE_SIZE_4KIB {
        let entry = usable_table[i];
        usable_table[i] = UsableMemoryEntry{address: entry.address, present: false}
    }
    //Create free memory area allocator
    let free_memory_area_allocator = UsableMemoryPageAllocator{table: oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut UsableMemoryEntry, len: usable_table.len(), identity_offset: oct4_to_usize(IDENTITY_OCT).unwrap()};
    {
        let free_memory_test = free_memory_area_allocator.allocate_page().unwrap();
        writeln!(printer, "Free Memory Area Allocation Test: {:?}", free_memory_test);
        writeln!(printer, "Free Memory Deallocation Test:    {:?}", free_memory_area_allocator.deallocate_page(free_memory_test));
        for i in 0..30 {
            let entry = usable_table[i];
            writeln!(printer, "{:2}: {}", i, entry.present);
        }
    }

    // PCI TESTING
    writeln!(printer, "\n=== PCI BUS ===\n");
    let mut pci_uhci_option = None;
    unsafe {for pci_bus in 0..256 {
        for pci_device in 0..32 {
            for pci_function in 0..8 {
                let pci_endpoint = match PciEndpoint::new(pci_bus, pci_device, pci_function) {Ok(pci) => pci, Err(_) => break};
                write!(printer, "PCI DEVICE:");
                write!(printer, "  Bus: {:02X}, Device: {:02X}, Function: {:01X}", pci_bus, pci_device, pci_function);
                writeln!(printer, "  |  Vendor ID: {:04X}, Device ID: {:04X}, Status: {:04X}", pci_endpoint.vendor_id(), pci_endpoint.device_id(), pci_endpoint.status());
                //writeln!(printer, "  Revision ID:   {:02X}, Prog IF:       {:02X}, Subclass:      {:02X}, Class Code:    {:02X}", pci.revision_id(), pci.prog_if(), pci.subclass(), pci.class_code());
                //writeln!(printer, "  Cache LSZ:     {:02X}, Latency Tmr:   {:02X}, Header Type:   {:02X}, BIST:          {:02X}", pci.chache_lz(), pci.latency(), pci.header_type(), pci.bist());
                if let Ok(o) = PciUhci::new(pci_endpoint) {pci_uhci_option = Some(o)};
            }
        }
    }}

    // USB TESTING
    unsafe {if let Some(mut pci_uhci) = pci_uhci_option {
        writeln!(printer, "\n=== UHCI USB === \n");
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

    // PS/2 KEYBOARD TESTING
    writeln!(printer, "\n=== PS/2 BUS===\n");
    let ps2_controller = Ps2Controller{};
    unsafe {
        ps2_controller.disable_port_1();
        ps2_controller.disable_port_2();
        let a1 = ps2_controller.read_memory(0x0000).unwrap();
        ps2_controller.write_memory(0, a1 & 0b1011_1100);
        if ps2_controller.test_controller() {
            writeln!(printer, "PS/2 Controller test succeeded.");
            let port_1 = ps2_controller.test_port_1();
            let port_2 = ps2_controller.test_port_2();
            if port_1 || port_2 {

            }
            else {writeln!(printer, "PS/2 Port tests failed.");}
        }
        else {writeln!(printer, "PS/2 Controller test failed.");}
    }

    // HALT COMPUTER
    panic!("\n=== IT IS NOW SAFE TO SHUT OFF YOUR COMPUTER ===")
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
        write!(printer, "\nKernel Halt: ");
        if let Some(panic_message) = panic_info.message() {
            writeln!(printer, "{}", panic_message);
        }
        if let Some(panic_location) = panic_info.location() {
            writeln!(printer, "File:   {}", panic_location.file());
            writeln!(printer, "Line:   {}", panic_location.line());
            writeln!(printer, "Column: {}", panic_location.column());
        }
        asm!("HLT");
        loop {}
    }
}


// MEMORY MANAGEMENT
#[derive(Clone, Copy)]
#[derive(Debug)]
struct UsableMemoryEntry {
    pub address: PhysicalAddress,
    pub present: bool,
}


struct NoneAllocator {
    pub identity_offset: usize
}
impl PageAllocator for NoneAllocator {
    fn allocate_page     (&self)                            -> Result<PhysicalAddress, &'static str> {
        Err("No Allocator: Allocate page called.")
    }
    fn deallocate_page   (&self, physical: PhysicalAddress) -> Result<(),              &'static str> {
        Err("No Allocator: De-allocate page called.")
    }
    fn physical_to_linear(&self, physical: PhysicalAddress) -> Result<LinearAddress,   &'static str> {
        Ok(LinearAddress(physical.add(self.identity_offset).0))
    }
}

struct UsableMemoryPageAllocator {
    pub table: *mut UsableMemoryEntry,
    pub len: usize,
    pub identity_offset: usize,
}
impl PageAllocator for UsableMemoryPageAllocator {
    fn allocate_page     (&self)                            -> Result<PhysicalAddress, &'static str> {
        for i in 0..self.len {
            let entry = unsafe {&mut *(self.table.add(i))};
            if entry.present {
                entry.present = false;
                return Ok(entry.address)
            }
        }
        Err("Usable Memory Page Allocator: Out of memory.")
    }

    fn deallocate_page   (&self, physical: PhysicalAddress) -> Result<(),              &'static str> {
        for i in 0..self.len {
            let entry = unsafe {&mut *(self.table.add(i))};
            if entry.address.0 == physical.0 {
                if !entry.present {
                    entry.present = true;
                    return Ok(())
                }
                else {
                    return Err("Usable Memory Page Allocator: Address to deallocate not allocated.")
                }
            }
        }
        Err("Usable Memory Page Allocator: Address to deallocate not found.")
    }

    fn physical_to_linear(&self, physical: PhysicalAddress) -> Result<LinearAddress,   &'static str> {
        Ok(LinearAddress(physical.add(self.identity_offset).0))
    }
}
