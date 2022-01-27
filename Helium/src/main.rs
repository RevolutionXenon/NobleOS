// HELIUM
// Helium is the Noble Kernel:
// Code execution
// Interrupt handling
// CPU time sharing
// (PLANNED) System call handling
// (PLANNED) Thread management
// (PLANNED) Program loading
// (PLANNED) Inter-process communication handling


// HEADER
//Flags
#![no_std]
#![no_main]
#![allow(unused_must_use)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::fn_to_numeric_cast)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(asm_sym)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(start)]

//Imports
mod gdt;
use photon::*;
use photon::formats::f2::*;
use gluon::GLUON_VERSION;
use gluon::noble::address_space::*;
use gluon::noble::input_events::*;
use gluon::pc::ports::*;
use gluon::x86_64::lapic;
use gluon::x86_64::pic;
use gluon::x86_64::ps2;
use gluon::x86_64::paging::*;
use gluon::x86_64::pci::*;
use gluon::x86_64::segmentation::*;
use core::convert::TryFrom;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::read_volatile;
use core::ptr::write_volatile;
use ::x86_64::instructions::hlt as halt;
use ::x86_64::instructions::interrupts::disable as cli;
use ::x86_64::instructions::interrupts::enable as sti;
use ::x86_64::registers::control::Cr3;
use ::x86_64::structures::idt::InterruptStackFrame;

//Constants
const HELIUM_VERSION: &str = "vDEV-2022-01-24"; //CURRENT VERSION OF KERNEL
static WHITESPACE:  CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK};
static _BLACKSPACE: CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLACK, background: COLOR_BGRX_WHITE};
static _BLUESPACE:  CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLUE,  background: COLOR_BGRX_BLACK};
static REDSPACE:    CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_RED,   background: COLOR_BGRX_BLACK};


// MACROS
//Interrupt that Panics (no error code)
macro_rules!interrupt_panic_noe {
    ($text:expr) => {{
        unsafe extern "x86-interrupt" fn handler(stack_frame: InterruptStackFrame) {
            asm!("MOV SS, {:x}", in(reg) u16::from(gdt::SUPERVISOR_DATA));
            asm!("MOV DS, {:x}", in(reg) u16::from(gdt::SUPERVISOR_DATA));
            panic!("\n{}\n{:?}\n", $text, stack_frame)
        }
        handler as unsafe extern "x86-interrupt" fn(InterruptStackFrame) as usize as u64
    }}
}

//Interrupt that panics (with error code)
macro_rules!interrupt_panic_err {
    ($text:expr) => {{
        unsafe extern "x86-interrupt" fn handler(stack_frame: InterruptStackFrame, error_code: u64) {
            asm!("MOV SS, {:x}", in(reg) u16::from(gdt::SUPERVISOR_DATA));
            asm!("MOV DS, {:x}", in(reg) u16::from(gdt::SUPERVISOR_DATA));
            panic!("\n{}\n{:?}\nERROR CODE: {:016X}\n", $text, stack_frame, error_code)
        }
        handler as unsafe extern "x86-interrupt" fn(InterruptStackFrame, u64) as usize as u64
    }}
}


// MAIN
//Main Entry Point After Hydrogen Boot
#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    // DISABLE INTERRUPTS
    cli();

    // GRAPHICS SETUP
    let pixel_renderer: PixelRendererHWD<ColorBGRX>;
    let character_renderer: CharacterTwoToneRenderer16x16<ColorBGRX>;
    let mut printer: PrintWindow::<PRINT_LINES, PRINT_HEIGHT, PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>;
    let mut inputter: InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>;
    {
        pixel_renderer = PixelRendererHWD {pointer: oct4_to_pointer(FRAME_BUFFER_OCT).unwrap() as *mut ColorBGRX, height: SCREEN_HEIGHT, width: SCREEN_WIDTH};
        character_renderer = CharacterTwoToneRenderer16x16::<ColorBGRX> {renderer: &pixel_renderer, height: FRAME_HEIGHT, width: FRAME_WIDTH, y: 0, x: 0};
        let mut frame: FrameWindow::<FRAME_HEIGHT, FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = FrameWindow::<FRAME_HEIGHT, FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, 0, 0);
        inputter = InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, INPUT_Y, INPUT_X);
        printer = PrintWindow::<PRINT_LINES, PRINT_HEIGHT, PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, WHITESPACE, WHITESPACE, PRINT_Y, PRINT_X);
        unsafe {GLOBAL_WRITE_POINTER = Some(&mut printer as &mut dyn Write as *mut dyn Write)};
        unsafe {GLOBAL_INPUT_POINTER = Some(&mut inputter as *mut InputWindow<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>)};
        //User Interface initialization
        frame.horizontal_line(PRINT_Y-1, 0, FRAME_WIDTH-1, REDSPACE);
        frame.horizontal_line(INPUT_Y-1, 0, FRAME_WIDTH-1, REDSPACE);
        frame.horizontal_line(INPUT_Y+1, 0, FRAME_WIDTH-1, REDSPACE);
        frame.vertical_line(0, PRINT_Y-1, INPUT_Y+1, REDSPACE);
        frame.vertical_line(FRAME_WIDTH-1, PRINT_Y-1, INPUT_Y+1, REDSPACE);
        frame.horizontal_string("NOBLE OS", 0, 0, REDSPACE);
        frame.horizontal_string("HELIUM KERNEL", 0, FRAME_WIDTH - 14 - HELIUM_VERSION.len(), REDSPACE);
        frame.horizontal_string(HELIUM_VERSION, 0, FRAME_WIDTH - HELIUM_VERSION.len(), REDSPACE);
        frame.render();
        writeln!(printer, "Welcome to Noble OS");
        writeln!(printer, "Helium Kernel           {}", HELIUM_VERSION);
        writeln!(printer, "Photon Graphics Library {}", PHOTON_VERSION);
        writeln!(printer, "Gluon Memory Library    {}", GLUON_VERSION);
    }

    // PAGE MAP PARSING
    writeln!(printer, "\n=== PAGE MAP ===\n");
    let none_alloc = NoneAllocator{identity_offset: oct4_to_usize(IDENTITY_OCT).unwrap()};
    let u_alloc: StackPageAllocator;
    let pml4: PageMap;
    {
        //Go to PML4
        let pml4_physical = PhysicalAddress(Cr3::read().0.start_address().as_u64() as usize);
        pml4 = PageMap::new(pml4_physical, PageMapLevel::L4, &none_alloc).unwrap();
        //Diagnostics
        writeln!(printer, "Physical Memory Area Present: {}", pml4.read_entry(PHYSICAL_OCT    ).unwrap().present);
        writeln!(printer, "Kernel Area Present:          {}", pml4.read_entry(KERNEL_OCT      ).unwrap().present);
        writeln!(printer, "Programs Area Present:        {}", pml4.read_entry(PROGRAMS_OCT    ).unwrap().present);
        writeln!(printer, "Frame Buffer Area Present:    {}", pml4.read_entry(FRAME_BUFFER_OCT).unwrap().present);
        writeln!(printer, "Free Memory Area Present:     {}", pml4.read_entry(FREE_MEMORY_OCT ).unwrap().present);
        writeln!(printer, "Offset Identity Area Present: {}", pml4.read_entry(IDENTITY_OCT    ).unwrap().present);
        writeln!(printer, "Page Map Area Present:        {}", pml4.read_entry(PAGE_MAP_OCT    ).unwrap().present);
        //Remove physical memory area
        let mut identity_not_present = pml4.read_entry(PHYSICAL_OCT).unwrap();
        identity_not_present.present = false;
        //pml4.write_entry(PHYSICAL_OCT, identity_not_present).unwrap();
        //Determine amount of free memory
        let free_page_count: usize = unsafe {read_volatile(oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *const u64)} as usize;
        writeln!(printer, "Free memory found: {}Pg or {}MiB {}KiB", free_page_count, (free_page_count*PAGE_SIZE_4KIB)/MIB, ((free_page_count*PAGE_SIZE_4KIB) % MIB)/KIB);
        u_alloc = StackPageAllocator {
            position: oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut usize,
            base_offset: unsafe {(oct4_to_pointer(FREE_MEMORY_OCT).unwrap() as *mut u64).add(1)},
            identity_offset: oct4_to_usize(IDENTITY_OCT).unwrap(),
        };
        //Test
        let test1 = u_alloc.allocate_page().unwrap();
        writeln!(printer, "{:016X}", test1.0);
        u_alloc.deallocate_page(test1);
        let test2 = u_alloc.allocate_page().unwrap();
        writeln!(printer, "{:016X}", test2.0);
    }

    // PCI TESTING
    writeln!(printer, "\n=== PERIPHERAL COMPONENT INTERCONNECT BUS ===\n");
    let mut pci_uhci_option = None;
    unsafe {
        //Iterate over pci busses
        for pci_bus in 0..256 {
            //Iterate over pci devices
            for pci_device in 0..32 {
                //Iterate over device functions
                for pci_function in 0..8 {
                    //Look at the current endpoint and skip if it doesn't exist
                    let pci_endpoint = match PciEndpoint::new(pci_bus, pci_device, pci_function) {Ok(pci) => pci, Err(_) => break};
                    //print diagnostics
                    write!(printer, "PCI DEVICE:");
                    write!(printer, "  Bus: {:02X}, Device: {:02X}, Function: {:01X}", pci_bus, pci_device, pci_function);
                    writeln!(printer, "  |  Vendor ID: {:04X}, Device ID: {:04X}, Status: {:04X}", pci_endpoint.vendor_id(), pci_endpoint.device_id(), pci_endpoint.status());
                    //writeln!(printer, "  Revision ID:   {:02X}, Prog IF:       {:02X}, Subclass:      {:02X}, Class Code:    {:02X}", pci.revision_id(), pci.prog_if(), pci.subclass(), pci.class_code());
                    //writeln!(printer, "  Cache LSZ:     {:02X}, Latency Tmr:   {:02X}, Header Type:   {:02X}, BIST:          {:02X}", pci.chache_lz(), pci.latency(), pci.header_type(), pci.bist());
                    //If a UHCI endpoint is found, keep track of it
                    if let Ok(o) = PciUhci::new(pci_endpoint) {pci_uhci_option = Some(o)};
                }
            }
        }
    }

    // USB TESTING
    unsafe {if let Some(mut pci_uhci) = pci_uhci_option {
        writeln!(printer, "\n=== UHCI USB ===\n");
        //PCI diagnostic
        for i in 0..0x0F {
            writeln!(printer, "PCI Register {:02X}: {:08X}", i, pci_uhci.pci.register(i).unwrap());
        }
        //USB diagnostic
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
    let gdt: GlobalDescriptorTable;
    writeln!(printer, "\n=== GLOBAL DESCRIPTOR TABLE ===\n");
    unsafe {
        //Allocate space for GDT
        gdt = GlobalDescriptorTable{address: u_alloc.physical_to_linear(u_alloc.allocate_page().unwrap()).unwrap(), limit: 512};
        writeln!(printer, "GDT Linear Address: 0x{:016X}", gdt.address.0);
        //Substitute symbol in TSS entry for actual TSS
        gdt::TASK_STATE_SEGMENT_ENTRY.base = &TASK_STATE_SEGMENT as *const TaskStateSegment as u64;
        //Write TSS entry into GDT
        gdt.write_system_entry(gdt::TASK_STATE_SEGMENT_ENTRY, gdt::TASK_STATE_SEGMENT_POSITION).unwrap();
        //Write GDT code and data entries
        gdt.write_entry(gdt::SUPERVISOR_CODE_ENTRY, gdt::SUPERVISOR_CODE_POSITION).unwrap();
        gdt.write_entry(gdt::SUPERVISOR_DATA_ENTRY, gdt::SUPERVISOR_DATA_POSITION).unwrap();
        gdt.write_entry(gdt::USER_CODE_ENTRY, gdt::USER_CODE_POSITION).unwrap();
        gdt.write_entry(gdt::USER_DATA_ENTRY, gdt::USER_DATA_POSITION).unwrap();
        //Load GDTR
        gdt.write_gdtr(gdt::SUPERVISOR_CODE, gdt::SUPERVISOR_DATA, gdt::SUPERVISOR_DATA);
        //Load Task Register
        asm!(
            "LTR {tsss:x}",
            tsss = in(reg) u16::from(gdt::TASK_STATE_SEGMENT_SELECTOR),
        );
    }

    // IDT SETUP
    writeln!(printer, "\n=== INTERRUPT DESCRIPTOR TABLE ===\n");
    let idt: InterruptDescriptorTable;
    {
        //Allocate space for IDT
        idt = InterruptDescriptorTable {address: u_alloc.physical_to_linear(u_alloc.allocate_page().unwrap()).unwrap(), limit: 255};
        //INT 00h - INT 19h
        //CPU exceptions
        let mut int_exception = InterruptDescriptor {
            offset: 0,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 1,
            descriptor_type: DescriptorType::InterruptGate,
        };
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x00 (#DE): Divide Error");                 idt.write_entry(&int_exception, 0x00);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x01 (#DB): Debug");                        idt.write_entry(&int_exception, 0x01);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x02: Non-Maskable Interrupt");             idt.write_entry(&int_exception, 0x02);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x03 (#BP): Breakpoint");                   idt.write_entry(&int_exception, 0x03);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x04 (#OF): Overflow");                     idt.write_entry(&int_exception, 0x04);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x05 (#BR): Bound Range Exceeded");         idt.write_entry(&int_exception, 0x05);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x06 (#UD): Invalid Opcode");               idt.write_entry(&int_exception, 0x06);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x07 (#NM): Device Not Available");         idt.write_entry(&int_exception, 0x07);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x08 (#DF): Double Fault");                 idt.write_entry(&int_exception, 0x08);
        int_exception.offset = interrupt_panic_err!("INTERRUPT VECTOR 0x0A (#TS): Invalid Task State Segment");   idt.write_entry(&int_exception, 0x0A);
        int_exception.offset = interrupt_panic_err!("INTERRUPT VECTOR 0x0B (#NP): Segment Not Present");          idt.write_entry(&int_exception, 0x0B);
        int_exception.offset = interrupt_panic_err!("INTERRUPT VECTOR 0x0C (#SS): Stack Fault");                  idt.write_entry(&int_exception, 0x0C);
        int_exception.offset = interrupt_panic_err!("INTERRUPT VECTOR 0x0D (#GP): General Protection Fault");     idt.write_entry(&int_exception, 0x0D);
        int_exception.offset = interrupt_panic_err!("INTERRUPT VECTOR 0x0E (#PF): Page Fault");                   idt.write_entry(&int_exception, 0x0E);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x10 (#MF): x87 FPU Floating Point Error"); idt.write_entry(&int_exception, 0x10);
        int_exception.offset = interrupt_panic_err!("INTERRUPT VECTOR 0x11 (#AC): Alignment Check");              idt.write_entry(&int_exception, 0x11);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x12 (#MC): Machine Check");                idt.write_entry(&int_exception, 0x12);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x13 (#XM): SIMD Floating Point Error");    idt.write_entry(&int_exception, 0x13);
        int_exception.offset = interrupt_panic_noe!("INTERRUPT VECTOR 0x14 (#VE): Virtualization Fault");         idt.write_entry(&int_exception, 0x14);
        int_exception.offset = interrupt_panic_err!("INTERRUPT VECTOR 0x15 (#CP): Control Protection Fault");     idt.write_entry(&int_exception, 0x15);
        //INT 20h - INT FFh
        //Immediate returns to all non-exception interrupts
        let int_user = InterruptDescriptor {
            offset: interrupt_immediate_return as unsafe extern "x86-interrupt" fn() as u64,
            segment_selector: gdt::USER_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 2,
            descriptor_type: DescriptorType::InterruptGate,
        };
        for position in 32..256 {idt.write_entry(&int_user, position);}
        //INT 21h
        //IRQ 1: Keyboard Handler
        let int_keyboard: InterruptDescriptor = InterruptDescriptor {
            offset: interrupt_irq_01 as unsafe extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 2,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_keyboard, 0x21);
        //INT 30h
        //LAPIC Timer
        let int_timer = InterruptDescriptor {
            offset: interrupt_timer as unsafe extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 2,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_timer, 0x30);
        //INT 80h
        //User accessible yield interrupt
        let int_yield = InterruptDescriptor {
            offset: interrupt_yield as unsafe extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::User,
            interrupt_stack_table: 2,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_yield, 0x80);
        //INT FFh
        //LAPIC Spurious Interrupt
        let int_spurious = InterruptDescriptor {
            offset: interrupt_spurious as extern "x86-interrupt" fn() as usize as u64,
            segment_selector: gdt::SUPERVISOR_CODE,
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 2,
            descriptor_type: DescriptorType::InterruptGate,
        };
        idt.write_entry(&int_spurious, 0xFF);
        //Write IDTR
        unsafe {idt.write_idtr();}
        //Diagnostic
        writeln!(printer, "IDT Linear Address: 0x{:016X}", idt.address.0);
    }

    // PIC SETUP
    writeln!(printer, "\n=== PROGRAMMABLE INTERRUPT CONTROLLER ===\n");
    unsafe {
        //Remap PIC
        pic::remap(0x20, 0x28).unwrap();
        //Enable IRQ 1
        pic::enable_irq(0x01).unwrap();
        //Test ability to read PIC inputs
        writeln!(printer, "{:08b} {:08b}", PORT_PIC1_DATA.read(), PORT_PIC2_DATA.read());
    }

    // PS/2 Bus
    writeln!(printer, "\n=== PERSONAL SYSTEM/2 BUS ===\n");
    unsafe {
        //Disable PS/2 ports
        ps2::disable_port1();
        ps2::disable_port2();
        //Set PS/2 flags
        let a1 = ps2::read_memory(0x0000).unwrap();
        ps2::write_memory(0x0000, a1 & 0b1011_1100).unwrap();
        //Test PS/2 controller
        let ps2_port1_present: bool;
        let ps2_port2_present: bool;
        if ps2::test_controller() {
            writeln!(printer, "PS/2 Controller test succeeded.");
            //Test PS/2 controller ports
            ps2_port1_present = ps2::test_port_1();
            ps2_port2_present = ps2::test_port_2();
            if ps2_port1_present || ps2_port2_present {
                writeln!(printer, "PS/2 Port tests succeeded:");
                //Enable Port 1
                if ps2_port1_present {
                    writeln!(printer, "  PS/2 Port 1 Present.");
                    ps2::flush_output();
                    ps2::keyboard_disable_scan().unwrap();
                    ps2::keyboard_set_scancode_set(1).unwrap();
                    let scancode_set = ps2::keyboard_get_scancode_set().unwrap();
                    writeln!(printer, "  PS/2 Keyboard Scancode Set: {}", scancode_set);
                    ps2::keyboard_enable_scan().unwrap();
                    ps2::enable_port1();
                    ps2::enable_int_port1();
                }
                //Enable Port 2
                if ps2_port2_present {
                    writeln!(printer, "  PS/2 Port 2 Present.");
                    //ps2::enable_port2();
                }
                else {
                    writeln!(printer, "  PS/2 Port 2 Not Present.");
                }
            }
            else {writeln!(printer, "PS/2 Port tests failed.");}
        }
        else {writeln!(printer, "PS/2 Controller test failed.");}
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
        pml3.map_pages_group_4kib(&group1, 0o001_000_000, true, true, true).unwrap();
        pml3.map_pages_group_4kib(&group2, 0o001_000_400, true, true, true).unwrap();
        pml3.map_pages_group_4kib(&group3, 0o001_001_000, true, true, true).unwrap();
        //Stack pointers
        let s1p = oct_to_usize_4(KERNEL_OCT, 1, 0, 0o377, 0).unwrap() as u64;
        let s2p = oct_to_usize_4(KERNEL_OCT, 1, 0, 0o777, 0).unwrap() as u64;
        let s3p = oct_to_usize_4(KERNEL_OCT, 1, 1, 0o377, 0).unwrap() as u64;
        //Instruction pointers
        let i1p = read_loop as fn() as usize as u64;
        let i2p = byte_loop as fn() as usize as u64;
        let i3p = ps2_keyboard as unsafe fn() as usize as u64;
        //Create tasks
        create_task(1, &u_alloc, i1p, gdt::SUPERVISOR_CODE, 0x00000202, s1p, gdt::SUPERVISOR_DATA);
        create_task(2, &u_alloc, i2p, gdt::USER_CODE, 0x00000202, s2p, gdt::USER_DATA);
        create_task(3, &u_alloc, i3p, gdt::USER_CODE, 0x00000202, s3p, gdt::USER_DATA);
        //Diagnostic
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
        //Diagnostic
        writeln!(printer, "APIC Present: {}", lapic::apic_check());
        writeln!(printer, "APIC Base: 0x{:16X}", lapic::get_base());
        //Shift Base
        lapic::LAPIC_ADDRESS = (lapic::get_base() as usize + oct4_to_usize(IDENTITY_OCT).unwrap()) as *mut u8;
        //Spurious interrupt
        lapic::spurious(0xFF);
        //Diagnostic
        writeln!(printer, "APIC ID:   0x{:1X}", lapic::read_register(0x20).unwrap() >> 24);
        writeln!(printer, "APIC 0xF0: 0x{:08X}", lapic::read_register(0xF0).unwrap());
        //Set Timer Mode
        lapic::timer(0x30, false, lapic::TimerMode::Periodic);
        lapic::divide_config(lapic::Divide::Divide_128);
        //Enable LAPIC
        lapic::enable();
    }

    // FINISH LOADING
    writeln!(printer, "\n=== STARTUP COMPLETE ===\n");
    unsafe {
        //Update TSS
        let kernel_stack = create_kernel_stack(0, &u_alloc) as u64;
        TASK_STATE_SEGMENT.ist1 = kernel_stack;
        TASK_STATE_SEGMENT.ist2 = kernel_stack;
        //Start LAPIC timer
        lapic::initial_count(100_000);
        //Enable Interrupts
        sti();
        //Halt init thread
        loop{halt();}
    }
}


// TASKING
//Global variables
static mut GLOBAL_WRITE_POINTER: Option<*mut dyn Write> = None;
static mut GLOBAL_INPUT_POINTER: Option<*mut InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>> = None;
static mut TASK_INDEX: usize = 0;
static mut TASK_STACKS: [u64; 4] = [0;4];

//Task Creation Function
unsafe fn create_task(thread_index: usize, allocator: &dyn PageAllocator, instruction_pointer: u64, code_selector: SegmentSelector, eflags_image: u32, stack_pointer: u64, stack_selector: SegmentSelector) {
    //Create stack
    let rsp = create_kernel_stack(thread_index, allocator);
    //Write stack frame
    write_volatile(rsp.sub(1), u16::from(stack_selector) as u64);
    write_volatile(rsp.sub(2), stack_pointer);
    write_volatile(rsp.sub(3), eflags_image as u64);
    write_volatile(rsp.sub(4), u16::from(code_selector) as u64);
    write_volatile(rsp.sub(5), instruction_pointer);
    //Zero register save states
    for i in 5..53 {
        write_volatile((stack_pointer as *mut u64).sub(i), 0);
    }
    //Save stack pointer
    TASK_STACKS[thread_index] = rsp.sub(52) as u64;
}

//Kernel Stack Allocation
unsafe fn create_kernel_stack(thread_index: usize, allocator: &dyn PageAllocator) -> *mut u64 {
    //Page map
    let pml4_physical = PhysicalAddress(Cr3::read().0.start_address().as_u64() as usize);
    let pml4 = PageMap::new(pml4_physical, PageMapLevel::L4, allocator).unwrap();
    let pml3 = PageMap::new(pml4.read_entry(STACKS_OCT).unwrap().physical, PageMapLevel::L3, allocator).unwrap();
    //Allocate stack
    let mut group = [PhysicalAddress(0);3];
    for address in &mut group {
        *address = allocator.allocate_page().unwrap();
    }
    pml3.map_pages_group_4kib(&group, (thread_index * 4) + 1, true, true, true).unwrap();
    //Initialize stack
    oct4_to_pointer(STACKS_OCT).unwrap().add((thread_index + 1) * 16 * KIB) as *mut u64
}

//Scheduler
unsafe extern "sysv64" fn scheduler() -> u64 {
    //Process thread to switch to
    let result = 
    if      INPUT_PIPE.state  == RingBufferState::WriteWait                                                    {TASK_INDEX = 3; TASK_STACKS[3]}
    else if STRING_PIPE.state == RingBufferState::WriteWait || STRING_PIPE.state == RingBufferState::ReadBlock {TASK_INDEX = 1; TASK_STACKS[1]}
    else                                                                                                       {TASK_INDEX = 0; TASK_STACKS[0]};
    //Change task state segment to new task
    TASK_STATE_SEGMENT.ist2 = (oct4_to_usize(STACKS_OCT).unwrap() + ((TASK_INDEX + 1) * 16 * KIB)) as u64;
    //Finish
    result
}


// THREADS
//Thread 1: Pipe Read and Print
static mut STRING_PIPE: RingBuffer<u8, 4096> = RingBuffer{data: [0xFF; 4096], read_head: 0, write_head: 0, state: RingBufferState::ReadWait};
fn read_loop() {unsafe {
    let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
    loop {
        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::ReadBlock);
        writeln!(printer, "{}", core::str::from_utf8(STRING_PIPE.read(&mut [0xFF; 4096])).unwrap());
        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::ReadWait);
        asm!("INT 80h");
    }
}}

//Thread 2: Pipe Write
fn byte_loop() {unsafe {
    loop {
        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::WriteBlock);
        STRING_PIPE.write("HELLO FROM USERSPACE! ".as_bytes());
        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::WriteWait);
        asm!("INT 80h");
    }
}}

//Thread 3: PS/2 Keyboard
static mut LEFT_SHIFT:  bool = false;
static mut RIGHT_SHIFT: bool = false;
static mut CAPS_LOCK:   bool = false;
static mut NUM_LOCK:    bool = false;
static mut INPUT_PIPE: RingBuffer<InputEvent, 512> = RingBuffer{data: [InputEvent{device_id: 0xFF, event_type: InputEventType::Blank, event_id: 0, event_data: 0}; 512], read_head: 0, write_head: 0, state: RingBufferState::Free};
unsafe fn ps2_keyboard() {
    let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
    let inputter: &mut InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = &mut *GLOBAL_INPUT_POINTER.unwrap();
    loop {
        write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::ReadBlock);
        let mut buffer = [InputEvent{device_id: 0xFF, event_type: InputEventType::Blank, event_id: 0, event_data: 0}; 512];
        let input_events = INPUT_PIPE.read(&mut buffer);
        for input_event in input_events {
            if input_event.event_type == InputEventType::DigitalKey {
                match KeyID::try_from(input_event.event_id) {Ok(key_id) => {
                    match PressType::try_from(input_event.event_data) {Ok(press_type) => {
                        match us_qwerty(key_id, CAPS_LOCK ^ (LEFT_SHIFT || RIGHT_SHIFT), NUM_LOCK) {
                            KeyStr::Key(key_id) => { match (key_id, press_type) {
                                (KeyID::NumLock,       PressType::Press)   => {NUM_LOCK    = !NUM_LOCK;}
                                (KeyID::KeyCapsLock,   PressType::Press)   => {CAPS_LOCK   = !CAPS_LOCK;}
                                (KeyID::KeyLeftShift,  PressType::Press)   => {LEFT_SHIFT  = true;}
                                (KeyID::KeyLeftShift,  PressType::Unpress) => {LEFT_SHIFT  = false;}
                                (KeyID::KeyRightShift, PressType::Press)   => {RIGHT_SHIFT = true;}
                                (KeyID::KeyRightShift, PressType::Unpress) => {RIGHT_SHIFT = false;}
                                _ => {}
                            }},
                            KeyStr::Str(s) => {match press_type {PressType::Press => {
                                for codepoint in s.chars() {
                                    if codepoint == '\n' {
                                        while STRING_PIPE.state == RingBufferState::ReadBlock {}
                                        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::WriteBlock);
                                        let mut buffer = [0u8; INPUT_LENGTH*4];
                                        let string = match inputter.to_str(&mut buffer) {
                                            Ok(string) => string,
                                            Err(error) => error,
                                        };
                                        STRING_PIPE.write(string.as_bytes());
                                        write_volatile(&mut STRING_PIPE.state as *mut RingBufferState, RingBufferState::WriteWait);
                                        inputter.flush(WHITESPACE);
                                    }
                                    else {
                                        inputter.push_render(CharacterTwoTone{codepoint, foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK}, WHITESPACE);
                                    }
                                }
                            } PressType::Unpress => {}}}
                        }
                    } Err(_) => {writeln!(printer, "Input Event Error: Unknown Press Type");}}
                } Err(_) => {writeln!(printer, "Input Event Error: Unknown Key ID");}}
            }
        }
        write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::Free);
        asm!("INT 80h");
    }
}


// INTERRUPT FUNCTIONS
//INT 20h-FFh: Immediate Return Interrupt
#[naked] unsafe extern "x86-interrupt" fn interrupt_immediate_return() {
    asm!("IRETQ", options(noreturn))
}

//INT 21h: PS/2 Keyboard IRQ
static mut PS2_SCANCODES: [u8;9] = [0u8;9];
static mut PS2_INDEX:   usize = 0x00;
unsafe extern "x86-interrupt" fn interrupt_irq_01() {
    while ps2::poll_output_buffer_status() {
        let scancode = ps2::read_output();
        PS2_SCANCODES[PS2_INDEX] = scancode;
        PS2_INDEX += 1;
        match ps2::scancodes_1(&PS2_SCANCODES[0..PS2_INDEX], 0x00) {
            Ok(ps2_scan) => match ps2_scan {
                ps2::Ps2Scan::Finish(input_event) => {
                    INPUT_PIPE.write(&[input_event]);
                    write_volatile(&mut INPUT_PIPE.state as *mut RingBufferState, RingBufferState::WriteWait);
                    PS2_INDEX = 0;
                }
                ps2::Ps2Scan::Continue => {}
            }
            Err(error) => {
                let printer = &mut *GLOBAL_WRITE_POINTER.unwrap();
                writeln!(printer, "PS/2 KEYBOARD ERROR: {:?} | {}", &PS2_SCANCODES[0..PS2_INDEX], error);
                PS2_INDEX = 0;
            }
        }
    }
    pic::end_irq(0x01).unwrap();
    //asm!("INT 80h");
}

//INT 30h: LAPIC Timer
#[naked] unsafe extern "x86-interrupt" fn interrupt_timer() {
    asm!(
        //Save Program State
        "PUSH RAX", "PUSH RBP", "PUSH R15", "PUSH R14",
        "PUSH R13", "PUSH R12", "PUSH R11", "PUSH R10",
        "PUSH R9",  "PUSH R8",  "PUSH RDI", "PUSH RSI",
        "PUSH RDX", "PUSH RCX", "PUSH RBX",
        //Save Extended State
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
                   "POP RBX", "POP RCX", "POP RDX",
        "POP RSI", "POP RDI", "POP R8",  "POP R9",
        "POP R10", "POP R11", "POP R12", "POP R13",
        "POP R14", "POP R15", "POP RBP", "POP RAX",
        //Enter code
        "IRETQ",
        //Symbols
        stack_array  = sym TASK_STACKS,
        stack_index  = sym TASK_INDEX,
        scheduler    = sym scheduler,
        lapic_eoi    = sym lapic::end_int,
        options(noreturn),
    )
}

//INT 80h: User Accessible CPU Yield
#[naked] unsafe extern "x86-interrupt" fn interrupt_yield() {
    asm!(
        //"UD2",
        //Save Program State
        "PUSH RAX", "PUSH RBP", "PUSH R15", "PUSH R14",
        "PUSH R13", "PUSH R12", "PUSH R11", "PUSH R10",
        "PUSH R9",  "PUSH R8",  "PUSH RDI", "PUSH RSI",
        "PUSH RDX", "PUSH RCX", "PUSH RBX",
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
                   "POP RBX", "POP RCX", "POP RDX",
        "POP RSI", "POP RDI", "POP R8",  "POP R9",
        "POP R10", "POP R11", "POP R12", "POP R13",
        "POP R14", "POP R15", "POP RBP", "POP RAX",
        //Enter code
        "IRETQ",
        //Symbols
        stack_array  = sym TASK_STACKS,
        stack_index  = sym TASK_INDEX,
        scheduler    = sym scheduler,
        options(noreturn),
    )
}

//INT FFh: LAPIC Spurious Interrupt
extern "x86-interrupt" fn interrupt_spurious() {unsafe {
    lapic::end_int()
}}

//Unused: Generic Interrupt
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
#[panic_handler]
unsafe extern "sysv64" fn panic_handler(panic_info: &PanicInfo) -> ! {
    cli();                                                            //Turn off interrupts
    if let Some(printer_pointer) = GLOBAL_WRITE_POINTER {             //Check for presence of write routines
        let printer = &mut *printer_pointer;                          //Find write routines
        write!(printer, "\nKernel Halt: ");                           //Begin writing
        if let Some(panic_message) = panic_info.message() {           //Check if there's an associated message
            writeln!(printer, "{}", panic_message);                   //Print the panic message
        }
        if let Some(panic_location) = panic_info.location() {         //Check if there's an associated source location
            writeln!(printer, "File:   {}", panic_location.file());   //Print the source file
            writeln!(printer, "Line:   {}", panic_location.line());   //Print the source line
            writeln!(printer, "Column: {}", panic_location.column()); //Print the source column
        }
    }
    loop {halt();};                                                   //Halt the processor
}


// MEMORY MANAGEMENT
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

struct StackPageAllocator {
    pub position:    *mut usize,
    pub base_offset: *mut u64,
    pub identity_offset:  usize,
}
impl PageAllocator for StackPageAllocator {
    fn allocate_page     (&self)                            -> Result<PhysicalAddress, &'static str> { unsafe {
        match read_volatile(self.position) {
            0 => Err("Stack Page Allocator: Out of memory."),
            position => Ok(PhysicalAddress({
                write_volatile(self.position, position-1);
                let address = read_volatile(self.base_offset.add(position-1)) as usize;
                let clear_pointer = (address + self.identity_offset) as *mut u8;
                for i in 0..PAGE_SIZE_4KIB {write_volatile(clear_pointer.add(i), 0);}
                address
            }))
        }
    }}

    fn deallocate_page   (&self, physical: PhysicalAddress) -> Result<(),              &'static str> {unsafe {
        write_volatile(self.base_offset.add(*self.position), physical.0 as u64);
        *self.position += 1;
        Ok(())
    }}

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


// TASK STATE SEGMENT
pub static mut TASK_STATE_SEGMENT: TaskStateSegment = TaskStateSegment {
    _0:    0,
    rsp0:  0,
    rsp1:  0,
    rsp2:  0,
    _1:    0,
    ist1:  0,
    ist2:  0,
    ist3:  0,
    ist4:  0,
    ist5:  0,
    ist6:  0,
    ist7:  0,
    _2:    0,
    _3:    0,
    iomba: 0,
};
