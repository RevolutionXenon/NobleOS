// HELIUM
// Helium is the Noble Kernel:
// (PLANNED) Program loading
// (PLANNED) Thread management
// Code execution
// CPU time sharing
// Interrupt handling
// (PLANNED) System call handling
// (PLANNED) Pipe management


// HEADER
//Flags
#![no_std]
#![no_main]
#![allow(unused_must_use)]
#![allow(clippy::if_same_then_else)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(start)]

//Imports
mod gdt;
use photon::*;
use gluon::*;
use gluon::x86_64_paging::*;
use gluon::x86_64_pci::*;
use gluon::x86_64_segmentation::*;
use x86_64::registers::control::Cr3;
use core::convert::TryFrom;
use core::{fmt::Write, ptr::{write_volatile}, slice::from_raw_parts_mut};
#[cfg(not(test))] use core::panic::PanicInfo;
#[cfg(not(test))] use x86_64::instructions::hlt as halt;
#[cfg(not(test))] use x86_64::instructions::interrupts::disable as cli;

use crate::gdt::SUPERVISOR_CODE;
use crate::gdt::SUPERVISOR_DATA;

//Constants
const HELIUM_VERSION: &str = "vDEV-2021-12-13"; //CURRENT VERSION OF KERNEL
static WHITESPACE:  CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK};
static _BLACKSPACE: CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLACK, background: COLOR_BGRX_WHITE};
static _BLUESPACE:  CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLUE,  background: COLOR_BGRX_BLACK};
static REDSPACE:    CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_RED,   background: COLOR_BGRX_BLACK};


//MACROS
//Interrupt that Panics
macro_rules! interrupt_panic {
    ($text:expr) => {{
        unsafe fn interrupt_handler() {
            asm!("MOV RSP, [{}+RIP]", sym TASK_STACKS);
            panic!($text);
        }
        interrupt_handler as usize as u64
    }}
}


// MAIN
//Main Entry Point After Hydrogen Boot
#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // GRAPHICS SETUP
    let pixel_renderer: PixelRendererHWD<ColorBGRX>;
    let character_renderer: CharacterTwoToneRenderer16x16<ColorBGRX>;
    let mut printer: PrintWindow::<F1_PRINT_LINES, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>;
    let mut inputter: InputWindow::<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>;
    {
        pixel_renderer = PixelRendererHWD {pointer: oct4_to_pointer(FRAME_BUFFER_OCT).unwrap() as *mut ColorBGRX, height: F1_SCREEN_HEIGHT, width: F1_SCREEN_WIDTH};
        character_renderer = CharacterTwoToneRenderer16x16::<ColorBGRX> {renderer: &pixel_renderer, height: F1_FRAME_HEIGHT, width: F1_FRAME_WIDTH, y: 0, x: 0};
        let mut frame: FrameWindow::<F1_FRAME_HEIGHT, F1_FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = FrameWindow::<F1_FRAME_HEIGHT, F1_FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, 0, 0);
        inputter = InputWindow::<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, F1_INPUT_Y, F1_INPUT_X);
        printer = PrintWindow::<F1_PRINT_LINES, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, WHITESPACE, F1_PRINT_Y, F1_PRINT_X);
        unsafe {GLOBAL_WRITE_POINTER = Some(&mut printer as &mut dyn Write as *mut dyn Write)};
        unsafe {GLOBAL_INPUT_POINTER = Some(&mut inputter as *mut InputWindow<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>)};
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
    }

    // PAGE MAP PARSING
    writeln!(printer, "\n=== PAGE MAP ===\n");
    let none_alloc = NoneAllocator{identity_offset: oct4_to_usize(IDENTITY_OCT).unwrap()};
    let u_alloc: UsableMemoryPageAllocator;
    let pml4: PageMap;
    {
        //Create "allocator" for address translation
        //Go to PML4
        let pml4_physical = PhysicalAddress(Cr3::read().0.start_address().as_u64() as usize);
        pml4 = PageMap::new(pml4_physical, PageMapLevel::L4, &none_alloc).unwrap();
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
        for entry in usable_table.iter_mut().take((free_page_count+PAGE_SIZE_4KIB-1)/PAGE_SIZE_4KIB) {
            *entry = UsableMemoryEntry{address: entry.address, present: false}
        }
        //Create free memory area allocator
        u_alloc = UsableMemoryPageAllocator{table: oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut UsableMemoryEntry, len: usable_table.len(), identity_offset: oct4_to_usize(IDENTITY_OCT).unwrap()};
        {
            let free_memory_test = u_alloc.allocate_page().unwrap();
            writeln!(printer, "Free Memory Area Allocation Test: {:?}", free_memory_test);
            writeln!(printer, "Free Memory Deallocation Test:    {:?}", u_alloc.deallocate_page(free_memory_test));
        }
    }

    // PCI TESTING
    writeln!(printer, "\n=== PERIPHERAL COMPONENT INTERCONNECT BUS ===\n");
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

    // GDT SETUP
    writeln!(printer, "\n=== GLOBAL DESCRIPTOR TABLE ===\n");
    {
        let gdt = GlobalDescriptorTable{address: u_alloc.physical_to_linear(u_alloc.allocate_page().unwrap()).unwrap(), limit: 512};
        writeln!(printer, "GDT Linear Address: 0x{:016X}", gdt.address.0);
        gdt.write_entry(gdt::SUPERVISOR_CODE_ENTRY, gdt::SUPERVISOR_CODE_POSITION).unwrap();
        gdt.write_entry(gdt::SUPERVISOR_DATA_ENTRY, gdt::SUPERVISOR_DATA_POSITION).unwrap();
        gdt.write_entry(gdt::USER_CODE_ENTRY, gdt::USER_CODE_POSITION).unwrap();
        gdt.write_entry(gdt::USER_DATA_ENTRY, gdt::USER_DATA_POSITION).unwrap();
        //Finalize
        unsafe {gdt.write_gdtr(gdt::SUPERVISOR_CODE, gdt::USER_DATA, gdt::SUPERVISOR_DATA);}
    }

    // IDT SETUP
    writeln!(printer, "\n=== INTERRUPT DESCRIPTOR TABLE ===\n");
    let idt = InterruptDescriptorTable {address: u_alloc.physical_to_linear(u_alloc.allocate_page().unwrap()).unwrap(), limit: 255};
    {
        writeln!(printer, "IDT Linear Address: 0x{:016X}", idt.address.0);
        //Generic Entry
        let mut idte = InterruptDescriptorTableEntry {
            offset: 0,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        //Write entries
        idte.offset = interrupt_panic!("Interrupt Vector 0x00 (#DE): Divide Error");                       idt.write_entry(&idte, 0x00);
        idte.offset = interrupt_panic!("Interrupt Vector 0x01 (#DB): Debug Exception");                    idt.write_entry(&idte, 0x01);
        idte.offset = interrupt_panic!("Interrupt Vector 0x03 (#BP): Breakpoint");                         idt.write_entry(&idte, 0x03);
        idte.offset = interrupt_panic!("Interrupt Vector 0x04 (#OF): Overflow");                           idt.write_entry(&idte, 0x04);
        idte.offset = interrupt_panic!("Interrupt Vector 0x05 (#BR): Bound Range Exceeded");               idt.write_entry(&idte, 0x05);
        idte.offset = interrupt_panic!("Interrupt Vector 0x06 (#UD): Invalid Opcode");                     idt.write_entry(&idte, 0x06);
        idte.offset = interrupt_panic!("Interrupt Vector 0x07 (#NM): No Floating Point Unit");             idt.write_entry(&idte, 0x07);
        idte.offset = interrupt_panic!("Interrupt Vector 0x08 (#DF): Double Fault");                       idt.write_entry(&idte, 0x08);
        idte.offset = interrupt_panic!("Interrupt Vector 0x0A (#TS): Invalid Task State Segment");         idt.write_entry(&idte, 0x0A);
        idte.offset = interrupt_panic!("Interrupt Vector 0x0B (#NP): Segment Not Present");                idt.write_entry(&idte, 0x0B);
        idte.offset = interrupt_panic!("Interrupt Vector 0x0C (#SS): Stack Segment Fault");                idt.write_entry(&idte, 0x0C);
        idte.offset = interrupt_panic!("Interrupt Vector 0x0D (#GP): General Protection Error");           idt.write_entry(&idte, 0x0D);
        idte.offset = interrupt_panic!("Interrupt Vector 0x0E (#PF): Page Fault");                         idt.write_entry(&idte, 0x0E);
        idte.offset = interrupt_panic!("Interrupt Vector 0x10 (#MF): Floating Point Math Fault");          idt.write_entry(&idte, 0x10);
        idte.offset = interrupt_panic!("Interrupt Vector 0x11 (#AC): Alignment Check");                    idt.write_entry(&idte, 0x11);
        idte.offset = interrupt_panic!("Interrupt Vector 0x12 (#MC): Machine Check");                      idt.write_entry(&idte, 0x12);
        idte.offset = interrupt_panic!("Interrupt Vector 0x13 (#XM): SIMD Floating Point Math Exception"); idt.write_entry(&idte, 0x13);
        idte.offset = interrupt_panic!("Interrupt Vector 0x14 (#VE): Virtualization Exception");           idt.write_entry(&idte, 0x14);
        idte.offset = interrupt_panic!("Interrupt Vector 0x15 (#CP): Control Protection Exception");       idt.write_entry(&idte, 0x15);
        //Test
        unsafe {idt.write_idtr();}
        //unsafe {asm!("ud2")} //test undefined opcode
        //unsafe {write_volatile(0x800000 as *mut u8, 0x00)} //test page fault
    }

    // PIC SETUP
    writeln!(printer, "\n=== PROGRAMMABLE INTERRUPT CONTROLLER ===\n");
    unsafe {
        x86_64_timers::pic_remap(0x20, 0x28).unwrap();
        x86_64_timers::pic_enable_irq(0x01).unwrap();
        let idte = InterruptDescriptorTableEntry {
            offset: irq_01_rust as unsafe extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&idte, 0x21);
        asm!("STI");
        //Test
        writeln!(printer, "{:08b} {:08b}", PORT_PIC1_DATA.read(), PORT_PIC2_DATA.read());
    }

    // PS/2 KEYBOARD
    writeln!(printer, "\n=== PERSONAL SYSTEM/2 BUS ===\n");
    unsafe {
        let ps2_port1_present: bool;
        let ps2_port2_present: bool;
        x86_64_ps2::disable_port1();
        x86_64_ps2::disable_port2();
        let a1 = x86_64_ps2::read_memory(0x0000).unwrap();
        x86_64_ps2::write_memory(0, a1 & 0b1011_1100).unwrap();
        if x86_64_ps2::test_controller() {
            writeln!(printer, "PS/2 Controller test succeeded.");
            ps2_port1_present = x86_64_ps2::test_port_1();
            ps2_port2_present = x86_64_ps2::test_port_2();
            if ps2_port1_present || ps2_port2_present {
                writeln!(printer, "PS/2 Port tests succeeded:");
                if ps2_port1_present {
                    writeln!(printer, "  PS/2 Port 1 Present.");
                    x86_64_ps2::enable_port1();
                    x86_64_ps2::enable_int_port1();
                }
                if ps2_port2_present {
                    writeln!(printer, "  PS/2 Port 2 Present.");
                    //ps2::enable_port2();
                }
            }
            else {writeln!(printer, "PS/2 Port tests failed.");}
        }
        else {writeln!(printer, "PS/2 Controller test failed.");}
    }

    // REGISTER TESTING
    writeln!(printer, "\n=== REGISTER TEST ===\n");
    {
        let mut cr3: u64;
        let mut rip: u64;
        unsafe {
            asm!("MOV {cr3}, CR3",   cr3 = out(reg) cr3, options(nostack));
            asm!("LEA {rip}, [RIP]", rip = out(reg) rip, options(nostack));
        }
        writeln!(printer, "CR3:    0x{:16X}", cr3);
        writeln!(printer, "RIP:    0x{:16X}", rip);
    }

    // CREATE THREADS
    writeln!(printer, "\n=== THREAD STACK TEST===\n");
    unsafe {
        //Do allocation
        const STACK_SIZE: usize = 0o377;
        let mut group1: [PhysicalAddress;STACK_SIZE] = [PhysicalAddress(0);STACK_SIZE];
        let mut group2: [PhysicalAddress;STACK_SIZE] = [PhysicalAddress(0);STACK_SIZE];
        let mut group3: [PhysicalAddress;STACK_SIZE] = [PhysicalAddress(0);STACK_SIZE];
        for i in 0..STACK_SIZE {
            group1[i] = u_alloc.allocate_page().unwrap();
            group2[i] = u_alloc.allocate_page().unwrap();
            group3[i] = u_alloc.allocate_page().unwrap();
        }
        //Edit page maps
        let pml3 = PageMap::new(pml4.read_entry(KERNEL_OCT).unwrap().physical, PageMapLevel::L3, &u_alloc).unwrap();
        pml3.map_pages_group_4kib(&group1, 0o001_000_000, true, false, true).unwrap();
        pml3.map_pages_group_4kib(&group2, 0o001_000_400, true, false, true).unwrap();
        pml3.map_pages_group_4kib(&group3, 0o001_001_000, true, false, true).unwrap();
        let s1p = oct_to_usize_4(KERNEL_OCT, 1, 0, 0o377, 0).unwrap() as u64;
        let s2p = oct_to_usize_4(KERNEL_OCT, 1, 0, 0o777, 0).unwrap() as u64;
        let s3p = oct_to_usize_4(KERNEL_OCT, 1, 1, 0o377, 0).unwrap() as u64;
        let i1p = read_loop as fn() as usize as u64;
        let i2p = byte_loop as fn() as usize as u64;
        let i3p = ps2_keyboard as fn() as usize as u64;
        TASK_STACKS[1] = create_task(i1p, gdt::SUPERVISOR_CODE, 0x00000202, s1p, gdt::SUPERVISOR_DATA);
        TASK_STACKS[2] = create_task(i2p, gdt::SUPERVISOR_CODE, 0x00000202, s2p, gdt::SUPERVISOR_DATA);
        TASK_STACKS[3] = create_task(i3p, gdt::USER_CODE, 0x00000202, s3p, gdt::USER_DATA);
        writeln!(printer, "Thread 1 (PIPE READ AND PRINT):");
        writeln!(printer, "  Stack Pointer Before Init: 0x{:16X}", s1p);
        writeln!(printer, "  Stack Pointer After Init:  0x{:16X}", TASK_STACKS[1]);
        writeln!(printer, "  Instruction Pointer:       0x{:16X}", i1p);
        writeln!(printer, "Thread 2 (PIPE WRITE):");
        writeln!(printer, "  Stack Pointer Before Init: 0x{:16X}", s2p);
        writeln!(printer, "  Stack Pointer After Init:  0x{:16X}", TASK_STACKS[2]);
        writeln!(printer, "  Instruction Pointer:       0x{:16X}", i2p);
        writeln!(printer, "Thread 3 (PS2 KEYBOARD):");
        writeln!(printer, "  Stack Pointer Before Init: 0x{:16X}", s3p);
        writeln!(printer, "  Stack Pointer After Init:  0x{:16X}", TASK_STACKS[3]);
        writeln!(printer, "  Instruction Pointer:       0x{:16X}", i3p);
    }

    // APIC SETUP
    writeln!(printer, "\n=== ADVANCED PROGRAMMABLE INTERRUPT CONTROLLER ===\n");
    unsafe {
        writeln!(printer, "APIC Present: {}", x86_64_timers::apic_check());
        writeln!(printer, "APIC Base: 0x{:16X}", x86_64_timers::lapic_get_base());
        x86_64_timers::LAPIC_ADDRESS = (x86_64_timers::lapic_get_base() as usize + oct4_to_usize(IDENTITY_OCT).unwrap()) as *mut u8;
        let mut idte = InterruptDescriptorTableEntry {
            offset: _interrupt_dummy as extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&idte, 0xFF);
        x86_64_timers::lapic_enable();
        x86_64_timers::lapic_spurious(0xFF);
        writeln!(printer, "APIC ID:   0x{:1X}", x86_64_timers::lapic_read_register(0x20).unwrap() >> 24);
        writeln!(printer, "APIC 0xF0: 0x{:08X}", x86_64_timers::lapic_read_register(0xF0).unwrap());
        idte.offset = interrupt_timer as unsafe extern "x86-interrupt" fn() as usize as u64;
        idt.write_entry(&idte, 0x30);
        x86_64_timers::lapic_timer(0x30, false, x86_64_timers::TimerMode::Periodic);
        x86_64_timers::lapic_divide_config(x86_64_timers::LapicDivide::Divide_128);
    }

    // FINISH LOADING
    writeln!(printer, "\n=== STARTUP COMPLETE ===\n");
    unsafe {x86_64_timers::lapic_initial_count(100_000);}
    loop {unsafe {asm!("HLT")}}
}


// TASKING
//Global variables
static mut GLOBAL_WRITE_POINTER: Option<*mut dyn Write> = None;
static mut GLOBAL_INPUT_POINTER: Option<*mut InputWindow::<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>> = None;
static mut TASK_INDEX: usize = 0;
static mut TASK_STACKS: [u64; 4] = [0;4];

//Task Creation Function
unsafe fn create_task(instruction_pointer: u64, code_selector: SegmentSelector, eflags_image: u32, stack_pointer: u64, stack_selector: SegmentSelector) -> u64 {
    write_volatile((stack_pointer as *mut u64).sub(1), u16::from(stack_selector) as u64);
    write_volatile((stack_pointer as *mut u64).sub(2), stack_pointer);
    write_volatile((stack_pointer as *mut u64).sub(3), eflags_image as u64);
    write_volatile((stack_pointer as *mut u64).sub(4), u16::from(code_selector) as u64);
    write_volatile((stack_pointer as *mut u64).sub(5), instruction_pointer);
    for i in 6..54 {
        write_volatile((stack_pointer as *mut u64).sub(i), 0);
    }
    stack_pointer - 424
}

//Scheduler
unsafe extern "sysv64" fn scheduler() -> u64 {
    if      INPUT_PIPE.state == RingBufferState::WriteWait || INPUT_PIPE.state == RingBufferState::ReadBlock  {TASK_INDEX = 3; TASK_STACKS[3]}
    else if TIMER_PIPE.state == RingBufferState::WriteWait || TIMER_PIPE.state == RingBufferState::ReadBlock  {TASK_INDEX = 1; TASK_STACKS[1]}
    else if TIMER_PIPE.state == RingBufferState::ReadWait  || TIMER_PIPE.state == RingBufferState::WriteBlock {TASK_INDEX = 2; TASK_STACKS[2]}
    else                                                                                                      {TASK_INDEX = 0; TASK_STACKS[0]}
}

//Pipe Testing Task
static mut TIMER_VAL: u8 = 0xFF;
static mut TIMER_PIPE: RingBuffer<u8, 4096> = RingBuffer{data: [0xFF; 4096], read_head: 0, write_head: 0, state: RingBufferState::ReadWait};
fn read_loop() {unsafe {
    let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
    loop {
        write_volatile(&mut TIMER_PIPE.state as *mut RingBufferState, RingBufferState::ReadBlock);
        write!(printer, "{}", core::str::from_utf8(TIMER_PIPE.read(&mut [0xFF; 4096])).unwrap());
        //writeln!(printer, "{:?}", TIMER_PIPE.read(&mut [0xFF; 4096])).unwrap();
        write_volatile(&mut TIMER_PIPE.state as *mut RingBufferState, RingBufferState::ReadWait);
        asm!("INT 30h");
    }
}}
fn byte_loop() {unsafe {
    loop {
        write_volatile(&mut TIMER_PIPE.state as *mut RingBufferState, RingBufferState::WriteBlock);
        TIMER_VAL = TIMER_VAL.wrapping_add(1);
        TIMER_PIPE.write("Hello World! ".as_bytes());
        //TIMER_PIPE.write(&[ps2::read_status()]);
        write_volatile(&mut TIMER_PIPE.state as *mut RingBufferState, RingBufferState::WriteWait);
        asm!("INT 30h");
    }
}}

//PS/2 Keyboard Task
static mut LEFT_SHIFT:  bool = false;
static mut RIGHT_SHIFT: bool = false;
static mut CAPS_LOCK:   bool = false;
static mut NUM_LOCK:    bool = false;
static mut INPUT_PIPE: RingBuffer<x86_64_ps2::InputEvent, 512> = RingBuffer{data: [x86_64_ps2::InputEvent{device_id: 0xFF, event_type: x86_64_ps2::InputEventType::Blank, event_id: 0, event_data: 0}; 512], read_head: 0, write_head: 0, state: RingBufferState::Free};
fn ps2_keyboard() {unsafe {
    let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
    let inputter: &mut InputWindow::<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = &mut *GLOBAL_INPUT_POINTER.unwrap();
    loop {
        write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::ReadBlock);
        let mut buffer = [x86_64_ps2::InputEvent{device_id: 0xFF, event_type: x86_64_ps2::InputEventType::Blank, event_id: 0, event_data: 0}; 512];
        let input_events = INPUT_PIPE.read(&mut buffer);
        for input_event in input_events {
            if input_event.event_type == x86_64_ps2::InputEventType::DigitalKey {
                match x86_64_ps2::KeyID::try_from(input_event.event_id) {Ok(key_id) => {
                    match x86_64_ps2::PressType::try_from(input_event.event_data) {Ok(press_type) => {
                        match x86_64_ps2::us_qwerty(key_id, CAPS_LOCK ^ (LEFT_SHIFT || RIGHT_SHIFT), NUM_LOCK) {
                            x86_64_ps2::KeyStr::Key(key_id) => { match (key_id, press_type) {
                                (x86_64_ps2::KeyID::NumLock,       x86_64_ps2::PressType::Press)   => {NUM_LOCK    = !NUM_LOCK;}
                                (x86_64_ps2::KeyID::KeyCapsLock,   x86_64_ps2::PressType::Press)   => {CAPS_LOCK   = !CAPS_LOCK;}
                                (x86_64_ps2::KeyID::KeyLeftShift,  x86_64_ps2::PressType::Press)   => {LEFT_SHIFT  = true;}
                                (x86_64_ps2::KeyID::KeyLeftShift,  x86_64_ps2::PressType::Unpress) => {LEFT_SHIFT  = false;}
                                (x86_64_ps2::KeyID::KeyRightShift, x86_64_ps2::PressType::Press)   => {RIGHT_SHIFT = true;}
                                (x86_64_ps2::KeyID::KeyRightShift, x86_64_ps2::PressType::Unpress) => {RIGHT_SHIFT = false;}
                                _ => {}
                            }},
                            x86_64_ps2::KeyStr::Str(s) => {match press_type {x86_64_ps2::PressType::Press => {
                                for codepoint in s.chars() {
                                    inputter.push_render(CharacterTwoTone{codepoint, foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK}, WHITESPACE);
                                }
                            } x86_64_ps2::PressType::Unpress => {}}}
                        }
                    } Err(_) => {writeln!(printer, "Input Event Error: Unknown Press Type");}}
                } Err(_) => {writeln!(printer, "Input Event Error: Unknown Key ID");}}
            }
        }
        write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::Free);
        asm!("INT 30h");
    }
}}


// INTERRUPT FUNCTIONS
//PS/2 Keyboard IRQ
static mut PS2_SCANCODES: [u8;9] = [0u8;9];
static mut PS2_INDEX:   usize = 0x00;
unsafe extern "x86-interrupt" fn irq_01_rust() {
    while x86_64_ps2::poll_input() {
        let scancode = x86_64_ps2::read_input();
        PS2_SCANCODES[PS2_INDEX] = scancode;
        PS2_INDEX += 1;
        match x86_64_ps2::scancodes_2(&PS2_SCANCODES[0..PS2_INDEX], 0x00) {
            Ok(ps2_scan) => match ps2_scan {
                x86_64_ps2::Ps2Scan::Finish(input_event) => {
                    INPUT_PIPE.write(&[input_event]);
                    write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::WriteWait);
                    PS2_INDEX = 0;
                }
                x86_64_ps2::Ps2Scan::Continue => {}
            }
            Err(error) => {
                let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
                writeln!(printer, "PS/2 KEYBOARD ERROR: {:?} | {}", &PS2_SCANCODES[0..PS2_INDEX], error);
                PS2_INDEX = 0;
            }
        }
    }
    x86_64_timers::pic_end_irq(0x01).unwrap();
}

//LAPIC Timer
#[naked] unsafe extern "x86-interrupt" fn interrupt_timer() {
    asm!(
        //Save Program State
        "PUSH RAX", "PUSH RBP", "PUSH R15", "PUSH R14",
        "PUSH R13", "PUSH R12", "PUSH R11", "PUSH R10",
        "PUSH R9",  "PUSH R8",  "PUSH RDI", "PUSH RSI",
        "PUSH RDX", "PUSH RCX", "PUSH RBX", "PUSH 0",
        "SUB RSP, 100h",
        "MOVAPS XMMWORD PTR [RSP + 0xf0], XMM15", "MOVAPS XMMWORD PTR [RSP + 0xe0], XMM14",
        "MOVAPS XMMWORD PTR [RSP + 0xd0], XMM13", "MOVAPS XMMWORD PTR [RSP + 0xc0], XMM12",
        "MOVAPS XMMWORD PTR [RSP + 0xb0], XMM11", "MOVAPS XMMWORD PTR [RSP + 0xa0], XMM10",
        "MOVAPS XMMWORD PTR [RSP + 0x90], XMM9",  "MOVAPS XMMWORD PTR [RSP + 0x80], XMM8",
        "MOVAPS XMMWORD PTR [RSP + 0x70], XMM7",  "MOVAPS XMMWORD PTR [RSP + 0x60], XMM6",
        "MOVAPS XMMWORD PTR [RSP + 0x50], XMM5",  "MOVAPS XMMWORD PTR [RSP + 0x40], XMM4",
        "MOVAPS XMMWORD PTR [RSP + 0x30], XMM3",  "MOVAPS XMMWORD PTR [RSP + 0x20], XMM2",
        "MOVAPS XMMWORD PTR [RSP + 0x10], XMM1",  "MOVAPS XMMWORD PTR [RSP + 0x00], XMM0",
        //Save stack pointer
        "MOV RAX, [{stack_index}+RIP]",
        "SHL RAX, 3",
        "LEA RCX, [{stack_array}+RIP]",
        "MOV [RCX+RAX], RSP",
        //Swap to kernel stack
        "MOV RSP, [{kernel_stack}+RIP]",
        //End interrupt
        "CALL {lapic_eoi}",
        //Call scheduler
        "CALL {scheduler}",
        //Swap to thread stack
        "MOV RSP, RAX",
        //Load program state
        "MOVAPS XMM0,  XMMWORD PTR [RSP + 0x00]", "MOVAPS XMM1,  XMMWORD PTR [RSP + 0x10]",
        "MOVAPS XMM2,  XMMWORD PTR [RSP + 0x20]", "MOVAPS XMM3,  XMMWORD PTR [RSP + 0x30]",
        "MOVAPS XMM4,  XMMWORD PTR [RSP + 0x40]", "MOVAPS XMM5,  XMMWORD PTR [RSP + 0x50]",
        "MOVAPS XMM6,  XMMWORD PTR [RSP + 0x60]", "MOVAPS XMM7,  XMMWORD PTR [RSP + 0x70]",
        "MOVAPS XMM8,  XMMWORD PTR [RSP + 0x80]", "MOVAPS XMM9,  XMMWORD PTR [RSP + 0x90]",
        "MOVAPS XMM10, XMMWORD PTR [RSP + 0xA0]", "MOVAPS XMM11, XMMWORD PTR [RSP + 0xB0]",
        "MOVAPS XMM12, XMMWORD PTR [RSP + 0xC0]", "MOVAPS XMM13, XMMWORD PTR [RSP + 0xD0]",
        "MOVAPS XMM14, XMMWORD PTR [RSP + 0xE0]", "MOVAPS XMM15, XMMWORD PTR [RSP + 0xF0]",
        "ADD RSP, 100h",
        "POP RBX", "POP RBX", "POP RCX", "POP RDX",
        "POP RSI", "POP RDI", "POP R8",  "POP R9",
        "POP R10", "POP R11", "POP R12", "POP R13",
        "POP R14", "POP R15", "POP RBP", "POP RAX",
        //Enter code
        "IRETQ",
        //Symbols
        stack_array  = sym TASK_STACKS,
        stack_index  = sym TASK_INDEX,
        kernel_stack = sym TASK_STACKS,
        scheduler    = sym scheduler,
        lapic_eoi    = sym x86_64_timers::lapic_end_int,
        options(noreturn),
    )
}

//Empty Function
extern "x86-interrupt" fn _interrupt_dummy() {unsafe {
    x86_64_timers::lapic_end_int()
}}

//Generic Interrupt (System Call?)
#[naked] unsafe extern "x86-interrupt" fn _interrupt_gen<const FP: u64>() {
    asm!(
        //Save Program State
        "PUSH RAX", "PUSH RBP", "PUSH R15", "PUSH R14",
        "PUSH R13", "PUSH R12", "PUSH R11", "PUSH R10",
        "PUSH R9",  "PUSH R8",  "PUSH RDI", "PUSH RSI",
        "PUSH RDX", "PUSH RCX", "PUSH RBX", "PUSH 0",
        "SUB RSP, 100h",
        "MOVAPS XMMWORD PTR [RSP + 0xf0], XMM15", "MOVAPS XMMWORD PTR [RSP + 0xe0], XMM14",
        "MOVAPS XMMWORD PTR [RSP + 0xd0], XMM13", "MOVAPS XMMWORD PTR [RSP + 0xc0], XMM12",
        "MOVAPS XMMWORD PTR [RSP + 0xb0], XMM11", "MOVAPS XMMWORD PTR [RSP + 0xa0], XMM10",
        "MOVAPS XMMWORD PTR [RSP + 0x90], XMM9",  "MOVAPS XMMWORD PTR [RSP + 0x80], XMM8",
        "MOVAPS XMMWORD PTR [RSP + 0x70], XMM7",  "MOVAPS XMMWORD PTR [RSP + 0x60], XMM6",
        "MOVAPS XMMWORD PTR [RSP + 0x50], XMM5",  "MOVAPS XMMWORD PTR [RSP + 0x40], XMM4",
        "MOVAPS XMMWORD PTR [RSP + 0x30], XMM3",  "MOVAPS XMMWORD PTR [RSP + 0x20], XMM2",
        "MOVAPS XMMWORD PTR [RSP + 0x10], XMM1",  "MOVAPS XMMWORD PTR [RSP + 0x00], XMM0",
        //Save stack pointer
        "MOV RAX, [{stack_index}+RIP]",
        "SHL RAX, 3",
        "LEA RCX, [{stack_array}+RIP]",
        "MOV [RCX+RAX], RSP",
        //Swap to kernel stack
        "MOV RSP, [{kernel_stack}+RIP]",
        //Call function
        "CALL {function_call}",
        //Reload stack
        "MOV RSP, RAX",
        //Load program state
        "MOVAPS XMM0,  XMMWORD PTR [RSP + 0x00]", "MOVAPS XMM1,  XMMWORD PTR [RSP + 0x10]",
        "MOVAPS XMM2,  XMMWORD PTR [RSP + 0x20]", "MOVAPS XMM3,  XMMWORD PTR [RSP + 0x30]",
        "MOVAPS XMM4,  XMMWORD PTR [RSP + 0x40]", "MOVAPS XMM5,  XMMWORD PTR [RSP + 0x50]",
        "MOVAPS XMM6,  XMMWORD PTR [RSP + 0x60]", "MOVAPS XMM7,  XMMWORD PTR [RSP + 0x70]",
        "MOVAPS XMM8,  XMMWORD PTR [RSP + 0x80]", "MOVAPS XMM9,  XMMWORD PTR [RSP + 0x90]",
        "MOVAPS XMM10, XMMWORD PTR [RSP + 0xA0]", "MOVAPS XMM11, XMMWORD PTR [RSP + 0xB0]",
        "MOVAPS XMM12, XMMWORD PTR [RSP + 0xC0]", "MOVAPS XMM13, XMMWORD PTR [RSP + 0xD0]",
        "MOVAPS XMM14, XMMWORD PTR [RSP + 0xE0]", "MOVAPS XMM15, XMMWORD PTR [RSP + 0xF0]",
        "ADD RSP, 100h",
        "POP RBX", "POP RBX", "POP RCX", "POP RDX",
        "POP RSI", "POP RDI", "POP R8",  "POP R9",
        "POP R10", "POP R11", "POP R12", "POP R13",
        "POP R14", "POP R15", "POP RBP", "POP RAX",
        //Enter code
        "IRETQ",
        //Symbols
        stack_array   = sym TASK_STACKS,
        stack_index   = sym TASK_INDEX,
        kernel_stack  = sym TASK_STACKS,
        function_call = const FP,
        options(noreturn),
    )
}


// PANIC HANDLER
#[cfg(not(test))]
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    unsafe {
        cli();                                                        //Turn off interrupts
        let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();            //Locate the global printer
        write!(printer, "\nKernel Halt: ");                           //Begin writing
        if let Some(panic_message) = panic_info.message() {           //Check if there's an associated message
            writeln!(printer, "{}", panic_message);                   //Print the panic message
        }
        if let Some(panic_location) = panic_info.location() {         //Check if there's an associated source location
            writeln!(printer, "File:   {}", panic_location.file());   //Print the source file
            writeln!(printer, "Line:   {}", panic_location.line());   //Print the source line
            writeln!(printer, "Column: {}", panic_location.column()); //Print the source column
        }
        loop {halt();};                                               //Halt the processor
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
    fn deallocate_page   (&self, _physical: PhysicalAddress) -> Result<(),              &'static str> {
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
                let linear = self.physical_to_linear(entry.address)?.0 as *mut u8;
                for i in 0..PAGE_SIZE_4KIB {
                    unsafe {write_volatile(linear.add(i), 0x00);}
                }
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


// PIPING
#[repr(C)]
struct RingBuffer<TYPE, const SIZE: usize> {
    state:       RingBufferState,
    read_head:   usize,
    write_head:  usize,
    data:        [TYPE; SIZE],
}
impl<TYPE, const SIZE: usize> RingBuffer<TYPE, SIZE> where TYPE: Clone+Copy {
    pub fn write(&mut self, data: &[TYPE]) {
        for item in data {
            self.data[self.write_head] = *item;
            self.write_head += 1;
            if self.write_head == SIZE {self.write_head = 0};
        }
    }
    pub fn read<'f>(&mut self, buffer: &'f mut [TYPE]) -> &'f [TYPE] {
        let mut j = 0;
        for item in &mut *buffer {
            if self.read_head == self.write_head {break}
            *item = self.data[self.read_head];
            j +=1;
            self.read_head += 1;
            if self.read_head == SIZE {self.read_head = 0}
        }
        &buffer[0..j]
    }
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum RingBufferState {
    Free = 0x00,
    WriteBlock = 0x01,
    WriteWait = 0x02,
    ReadBlock = 0x03,
    ReadWait = 0x04,
}
