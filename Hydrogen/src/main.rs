// HYDROGEN
// Hydrogen is the Noble bootloader:
// Memory and control register diagnostics
// Kernel space binary loading
// Virtual memory initialization
// Kernel booting


// HEADER
//Flags
#![no_std]
#![no_main]
#![allow(unused_must_use)]
#![allow(clippy::single_char_pattern)]
#![feature(abi_efiapi)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(asm_sym)]
#![feature(bench_black_box)]
#![feature(box_syntax)]

//External crates
extern crate rlibc;
extern crate alloc;


//Imports
use photon::*;
use photon::formats::f2::*;
use gluon::*;
use gluon::sysv_executable::*;
use gluon::x86_64_paging::*;
use gluon::x86_64_segmentation::*;
use core::cell::RefCell;
use core::convert::TryInto;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr;
use core::ptr::read_volatile;
use core::ptr::write_volatile;
use core::str::Split;
use uefi::prelude::*;
use uefi::proto::console::gop::*;
use uefi::proto::console::text::*;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::*;
use uefi::table::boot::*;
use uefi::table::runtime::*;
use x86_64::registers::control::*;
use x86_64::structures::idt::InterruptStackFrame;

//Constants
const HYDROGEN_VERSION: &str = "vDEV-2022-01-12"; //CURRENT VERSION OF BOOTLOADER


// MACROS
//Interrupt that Panics
macro_rules! interrupt_panic_noe {
    ($text:expr) => {{
        unsafe extern "x86-interrupt" fn interrupt_handler(stack_frame: InterruptStackFrame) {
            panic!($text, stack_frame);
        }
        interrupt_handler as usize as u64
    }}
}

macro_rules! interrupt_panic_err {
    ($text:expr) => {{
        unsafe extern "x86-interrupt" fn interrupt_handler(stack_frame: InterruptStackFrame, error_code: u64) {
            let printer = &mut *PANIC_WRITE_POINTER.unwrap();
            for i in 0..10 {
                let a = read_volatile((stack_frame.stack_pointer.as_ptr() as *const u64).add(i));
                writeln!(printer, "{:016X}", a);
            }
            panic!($text, stack_frame, error_code);
        }
        interrupt_handler as usize as u64
    }}
}

// MAIN
//Entry Point After UEFI Boot
#[entry]
fn efi_main(handle: Handle, system_table_boot: SystemTable<Boot>) -> Status {
    boot_main(handle, system_table_boot)
}

//Main Function
fn boot_main(handle: Handle, mut system_table_boot: SystemTable<Boot>) -> Status {
    // UEFI INITILIZATION
    //Console Reset
    system_table_boot.stdout().reset(false).expect("Console reset failed.");
    //Utilities Initialization (Alloc)
    uefi_services::init(&mut system_table_boot).expect("UEFI Initialization: failed at utilities initialization.");
    let boot_services = system_table_boot.boot_services();
    //Watchdog Timer Shutoff
    boot_services.set_watchdog_timer(0, 0x10000, Some(&mut {&mut [0x0058u16, 0x0000u16]}[..])).expect("UEFI Initialization: Watchdog Timer shutoff failed.");
    //Graphics Output Initialization
    let graphics_output_protocol = unsafe{&mut *match boot_services.locate_protocol::<GraphicsOutput>() {
        Ok(completion) => completion, 
        Err(error) => panic!("Graphics Output Initialization: Failed due to {}", uefi_error_readout(error.status()))}
    .expect("Graphics Output Initialization: Failed due to Unsafe Cell expect.").get()};
    //Screen Variables
    let graphics_frame_pointer: *mut u8 = graphics_output_protocol.frame_buffer().as_mut_ptr();
    let whitespace: CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK};
    let _blackspace: CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLACK, background: COLOR_BGRX_WHITE};
    let bluespace: CharacterTwoTone::<ColorBGRX> = CharacterTwoTone::<ColorBGRX> {codepoint: ' ', foreground: COLOR_BGRX_BLUE, background: COLOR_BGRX_BLACK};
    let pixel_renderer: PixelRendererHWD<ColorBGRX> = PixelRendererHWD {pointer: graphics_frame_pointer as *mut ColorBGRX, height: SCREEN_HEIGHT, width: SCREEN_WIDTH};
    let character_renderer: CharacterTwoToneRenderer16x16<ColorBGRX> = CharacterTwoToneRenderer16x16::<ColorBGRX> {renderer: &pixel_renderer, height: FRAME_HEIGHT, width: FRAME_WIDTH, y: 0, x: 0};
    let mut frame: FrameWindow::<FRAME_HEIGHT, FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = FrameWindow::<FRAME_HEIGHT, FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, whitespace, 0, 0);
    let mut printer: PrintWindow::<PRINT_LINES, PRINT_HEIGHT, PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = PrintWindow::<PRINT_LINES, PRINT_HEIGHT, PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, whitespace, whitespace, PRINT_Y, PRINT_X);
    let mut inputter: InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>  = InputWindow::<INPUT_LENGTH, INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, whitespace, INPUT_Y, INPUT_X);
    //Panic printer
    unsafe {PANIC_WRITE_POINTER = Some(&mut printer as &mut dyn Write as *mut dyn Write)};
    //Graphics Output Protocol: Set Graphics Mode
    uefi_set_graphics_mode(graphics_output_protocol);
    writeln!(printer, "PIXEL STRIDE {:?}", graphics_output_protocol.current_mode_info().stride());
    let _pf = graphics_output_protocol.current_mode_info().pixel_format();
    let _s = graphics_output_protocol.frame_buffer().size();
    //Simple File System Initialization
    let simple_file_system = unsafe{&mut *match boot_services.locate_protocol::<SimpleFileSystem>() {
        Ok(completion) => completion, 
        Err(error) => panic!("Simple File System Initialization: Failed due to {}", uefi_error_readout(error.status()))}
    .expect("Simple File System Initialization: Failed due to Unsafe Cell expect.").get()};
    //Input Initialization
    let input = unsafe{&mut *match boot_services.locate_protocol::<Input>() {
        Ok(completion) => completion, 
        Err(error) => panic!("Input Initialization: Failed due to {}", uefi_error_readout(error.status()))}
    .expect("Input Initialization: Failed due to Unsafe Cell expect.").get()};

    // GRAPHICS SETUP
    //User Interface initialization
    frame.horizontal_line(PRINT_Y-1,     0,         FRAME_WIDTH-1,  bluespace);
    frame.horizontal_line(INPUT_Y-1,     0,         FRAME_WIDTH-1,  bluespace);
    frame.horizontal_line(INPUT_Y+1,     0,         FRAME_WIDTH-1,  bluespace);
    frame.vertical_line(  0,             PRINT_Y-1, INPUT_Y+1,      bluespace);
    frame.vertical_line(  FRAME_WIDTH-1, PRINT_Y-1, INPUT_Y+1,      bluespace);
    frame.horizontal_string("NOBLE OS",            0, 0,                                         bluespace);
    frame.horizontal_string("HYDROGEN BOOTLOADER", 0, FRAME_WIDTH - 20 - HYDROGEN_VERSION.len(), bluespace);
    frame.horizontal_string(HYDROGEN_VERSION,      0, FRAME_WIDTH -      HYDROGEN_VERSION.len(), bluespace);
    frame.render();
    writeln!(printer, "\n=== WELCOME TO NOBLE OS ===\n");
    writeln!(printer, "Hydrogen Bootloader     {}", HYDROGEN_VERSION);
    writeln!(printer, "Photon Graphics Library {}", PHOTON_VERSION);
    writeln!(printer, "Gluon Boot Library      {}", GLUON_VERSION);

    // LOAD KERNEL
    //Find kernel on disk
    writeln!(printer, "\n=== LOADING KERNEL ===\n");
    let mut sfs_dir_root = simple_file_system.open_volume().expect_success("File system root failed to open.");
    let sfs_kernel_handle = sfs_dir_root.open("noble", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"noble\".").
        open("helium", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"helium\".").
        open("x86-64.elf", FileMode::Read, FileAttribute::empty()).expect_success("File system kernel open failed at \"x86-64.elf\".");
    let mut sfs_kernel = unsafe {RegularFile::new(sfs_kernel_handle)};
    writeln!(printer, "Found kernel on file system.");
    //Read kernel file
    let sfs_kernel_wrap = LocationalReadWrapper{ref_cell: RefCell::new(&mut sfs_kernel)};
    let mut kernel = match ELFFile::new(&sfs_kernel_wrap) {
        Ok(elffile) =>  elffile,
        Err(error) => panic!("{}", error),
    };
    //Check ELF header validity
    if kernel.header.binary_interface         != ApplicationBinaryInterface::None     {writeln!(printer, "Kernel load: Incorrect Application Binary Interface (ei_osabi). Should be SystemV/None (0x00)."); panic!();}
    if kernel.header.binary_interface_version != 0x00                                 {writeln!(printer, "Kernel load: Incorrect Application Binary Interface Version (ei_abiversion). Should be None (0x00)."); panic!();}
    if kernel.header.architecture             != InstructionSetArchitecture::EmX86_64 {writeln!(printer, "Kernel load: Incorrect Instruction Set Architecture (e_machine). Should be x86-64 (0x3E)."); panic!();}
    if kernel.header.object_type              != ObjectType::Shared                   {writeln!(printer, "Kernel Load: Incorrect Object Type (e_type). Should be Dynamic (0x03)."); panic!()}
    //Allocate memory for kernel
    let kernel_size: usize = kernel.program_memory_size() as usize;
    let kernel_location: *mut u8 = unsafe {allocate_memory(boot_services, MemoryType::LOADER_CODE, kernel_size, PAGE_SIZE_4KIB)};
    let stack_size: usize = 8*MIB;
    let stack_location: *mut u8 = unsafe {allocate_memory(boot_services, MemoryType::LOADER_CODE, stack_size, PAGE_SIZE_4KIB)};
    //Load kernel into memory
    unsafe {
        match kernel.load(kernel_location) {
            Ok(()) => {
                writeln!(printer, "Kernel successfully loaded at:    0x{:16X}", kernel_location as usize);
                writeln!(printer, "Kernel stack located at:          0x{:16X}", stack_location as usize);
                match kernel.relocate(kernel_location, oct4_to_pointer(KERNEL_OCT).unwrap()) {
                    Ok(()) => {writeln!(printer, "Kernel successfully relocated to: 0x{:16X}", oct4_to_pointer(KERNEL_OCT).unwrap() as usize);},
                    Err(error) => {writeln!(printer, "{}", error);},
                }
            }
            Err(error) => {writeln!(printer, "{}", error);}
        }
    }
    //Print ELF header info
    writeln!(printer, "\n=== KERNEL INFO ===\n");
    writeln!(printer, "Kernel Entry Point:                 0x{:16X}", kernel.header.entry_point);
    writeln!(printer, "Kernel Program Header Offset:       0x{:16X}", kernel.header.program_header_offset);
    writeln!(printer, "Kernel Section Header Offset:       0x{:16X}", kernel.header.section_header_offset);
    writeln!(printer, "Kernel ELF Header Size:             0x{:16X}", kernel.header.header_size);
    writeln!(printer, "Kernel Program Header Entry Size:   0x{:16X}", kernel.header.program_header_entry_size);
    writeln!(printer, "Kernel Program Header Number:       0x{:16X}", kernel.header.program_header_number);
    writeln!(printer, "Kernel Section Header Entry Size:   0x{:16X}", kernel.header.section_header_entry_size);
    writeln!(printer, "Kernel Section Header Number:       0x{:16X}", kernel.header.section_header_number);
    writeln!(printer, "Kernel Section Header String Index: 0x{:16X}", kernel.header.string_section_index);
    writeln!(printer, "Kernel Code Size:                   0x{:16X} or {}KiB", kernel_size, kernel_size / KIB);

    // MEMORY MAP SETUP
    let pml4: PageMap;
    let pml3_free_memory: PageMap;
    let page_allocator = UefiPageAllocator{boot_services};
    unsafe {
        // NX BIT
        let mut val = Efer::read().bits();
        val |= 0x800;
        Efer::write_raw(val);
        // NEW PAGE MAP SYSTEM
        writeln!(printer, "\n=== NEW PAGE MAP SYSTEM ===\n");
        //Page Map Level 4: OS Boot
        pml4                                  = PageMap::new(allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA), PageMapLevel::L4, &page_allocator).unwrap();
        writeln!(printer, "PML4 KNENV: 0x{:16X}", pml4.linear.0);
        //Page Map Level 4: EFI Boot
        let pml4_efi_boot:            PageMap = PageMap::new(PhysicalAddress(Cr3::read().0.start_address().as_u64() as usize), PageMapLevel::L4, &page_allocator).unwrap();
        writeln!(printer, "PML4 EFIBT: 0x{:16X}", pml4_efi_boot.linear.0);
        //Page Map Level 3: EFI Boot Physical Memory
        let pml3_efi_physical_memory: PageMap = PageMap::new(pml4_efi_boot.read_entry(0).unwrap().physical, PageMapLevel::L3, &page_allocator).unwrap();
        writeln!(printer, "PML3 EFIPH: 0x{:16X}", pml3_efi_physical_memory.linear.0);
        //Page Map Level 3: Kernel
        let pml3_kernel:              PageMap = PageMap::new(allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA), PageMapLevel::L3, &page_allocator).unwrap();
            pml3_kernel               .map_pages_offset_4kib(PhysicalAddress(kernel_location as usize), 0,                           kernel_size, true, true, false).unwrap();
            pml3_kernel               .map_pages_offset_4kib(PhysicalAddress(stack_location  as usize), PAGE_SIZE_512G - stack_size, stack_size,  true, true, true ).unwrap();
        writeln!(printer, "PML3 KERNL: 0x{:16X}", pml3_kernel.linear.0);
        //Page Map Level 3: Kernel Stacks
        let pml3_kernel_stacks:       PageMap = PageMap::new(allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA), PageMapLevel::L3, &page_allocator).unwrap();
        writeln!(printer, "PML3 KSTAK: 0x{:16X}", pml3_kernel_stacks.linear.0);
        //Page Map Level 3: Frame Buffer
        let pml3_frame_buffer:        PageMap = PageMap::new(allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA), PageMapLevel::L3, &page_allocator).unwrap();
            pml3_frame_buffer         .map_pages_offset_4kib(PhysicalAddress(graphics_frame_pointer as usize), 0, SCREEN_HEIGHT*SCREEN_WIDTH*4, true, true, true).unwrap();
        writeln!(printer, "PML3 FRAME: 0x{:16X}", pml3_frame_buffer.linear.0);
        //Page Map Level 3: Free Memory
            pml3_free_memory                  = PageMap::new(allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA), PageMapLevel::L3, &page_allocator).unwrap();
        //Page Map Level 3: Offset Identity Map
        let pml3_offset_identity_map: PageMap = PageMap::new(allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA), PageMapLevel::L3, &page_allocator).unwrap();
            pml3_offset_identity_map  .map_pages_offset_1gib(PhysicalAddress(0), 0, PAGE_SIZE_512G, true, true, true).unwrap();
        writeln!(printer, "PML3 IDENT: 0x{:16X}", pml3_offset_identity_map.linear.0);
        //Write PML4 Entries
        pml4.write_entry(PHYSICAL_OCT,     PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_efi_physical_memory.physical, true, true, false).unwrap()).unwrap();
        pml4.write_entry(KERNEL_OCT,       PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_kernel             .physical, true, true, false).unwrap()).unwrap();
        pml4.write_entry(STACKS_OCT,       PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_kernel_stacks      .physical, true, false,false).unwrap()).unwrap();
        pml4.write_entry(FRAME_BUFFER_OCT, PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_frame_buffer       .physical, true, true, true ).unwrap()).unwrap();
        pml4.write_entry(FREE_MEMORY_OCT,  PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_free_memory        .physical, true, true, true ).unwrap()).unwrap();
        pml4.write_entry(IDENTITY_OCT,     PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_offset_identity_map.physical, true, true, true ).unwrap()).unwrap();
        pml4.write_entry(PAGE_MAP_OCT,     PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml4                    .physical, true, true, true ).unwrap()).unwrap();
        // FINISH
    }

    // IDT SETUP
    writeln!(printer, "\n=== SYSTEM TABLES ===\n");
    let idt: InterruptDescriptorTable;
    unsafe {
        //Locate GDT and IDT
        let idtr = [0u8;10];
        let gdtr = [0u8;10];
        let mut tr: u16;
        asm!("SIDT [{}]", in(reg) &idtr);
        asm!("SGDT [{}]", in(reg) &gdtr);
        asm!("STR {:x}", out(reg) tr);
        let idt_address = u64::from_le_bytes(idtr[2..10].try_into().unwrap());
        let gdt_address = u64::from_le_bytes(gdtr[2..10].try_into().unwrap());
        //Diagnostic
        writeln!(printer, "GDT Linear Address: 0x{:016X}", gdt_address);
        writeln!(printer, "IDT Linear Address: 0x{:016X}", idt_address);
        writeln!(printer, "Task Register: 0x{:04X}", tr);
        //Find long mode entry in gdt
        let gdt_pointer = gdt_address as *mut u64;
        let mut code_index: u16 = 0;
        for i in 0..255 {
            let test = *(gdt_pointer.add(i));
            let b1 = test&(1<<53) != 0;
            let b2 = test&(3<<45) == 0;
            //let b2 = true;
            if b1 && b2 {
                code_index = i as u16;
                writeln!(printer, "CODE ENTRY FOUND AT POSITION {}", i);
                break;
            }
        }
        //Setup idt
        idt = InterruptDescriptorTable {address: LinearAddress(idt_address as usize), limit: 255};
        //Generic Entry
        let mut idte = InterruptDescriptorTableEntry {
            offset: 0,
            segment_selector: SegmentSelector {descriptor_table_index: code_index, table_indicator: TableIndicator::GDT, requested_privilege_level: PrivilegeLevel::Supervisor },
            segment_present: true,
            privilege_level: PrivilegeLevel::Supervisor,
            interrupt_stack_table: 0,
            descriptor_type: DescriptorType::InterruptGate,
        };
        //Write entries for CPU exceptions
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x00 (#DE): Divide Error                      \n{:?}\n");                       idt.write_entry(&idte, 0x00);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x01 (#DB): Debug Exception                   \n{:?}\n");                       idt.write_entry(&idte, 0x01);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x03 (#BP): Breakpoint                        \n{:?}\n");                       idt.write_entry(&idte, 0x03);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x04 (#OF): Overflow                          \n{:?}\n");                       idt.write_entry(&idte, 0x04);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x05 (#BR): Bound Range Exceeded              \n{:?}\n");                       idt.write_entry(&idte, 0x05);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x06 (#UD): Invalid Opcode                    \n{:?}\n");                       idt.write_entry(&idte, 0x06);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x07 (#NM): No Floating Point Unit            \n{:?}\n");                       idt.write_entry(&idte, 0x07);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x08 (#DF): Double Fault                      \n{:?}\n");                       idt.write_entry(&idte, 0x08);
        idte.offset = interrupt_panic_err!("\nInterrupt Vector 0x0A (#TS): Invalid Task State Segment        \n{:?}\nError Code 0x{:016X}\n"); idt.write_entry(&idte, 0x0A);
        idte.offset = interrupt_panic_err!("\nInterrupt Vector 0x0B (#NP): Segment Not Present               \n{:?}\nError Code 0x{:016X}\n"); idt.write_entry(&idte, 0x0B);
        idte.offset = interrupt_panic_err!("\nInterrupt Vector 0x0C (#SS): Stack Segment Fault               \n{:?}\nError Code 0x{:016X}\n"); idt.write_entry(&idte, 0x0C);
        idte.offset = interrupt_panic_err!("\nInterrupt Vector 0x0D (#GP): General Protection Error          \n{:?}\nError Code 0x{:016X}\n"); idt.write_entry(&idte, 0x0D);
        idte.offset = interrupt_panic_err!("\nInterrupt Vector 0x0E (#PF): Page Fault                        \n{:?}\nError Code 0x{:016X}\n"); idt.write_entry(&idte, 0x0E);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x10 (#MF): Floating Point Math Fault         \n{:?}\n");                       idt.write_entry(&idte, 0x10);
        idte.offset = interrupt_panic_err!("\nInterrupt Vector 0x11 (#AC): Alignment Check                   \n{:?}\nError Code 0x{:016X}\n"); idt.write_entry(&idte, 0x11);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x12 (#MC): Machine Check                     \n{:?}\n");                       idt.write_entry(&idte, 0x12);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x13 (#XM): SIMD Floating Point Math Exception\n{:?}\n");                       idt.write_entry(&idte, 0x13);
        idte.offset = interrupt_panic_noe!("\nInterrupt Vector 0x14 (#VE): Virtualization Exception          \n{:?}\n");                       idt.write_entry(&idte, 0x14);
        idte.offset = interrupt_panic_err!("\nInterrupt Vector 0x15 (#CP): Control Protection Exception      \n{:?}\nError Code 0x{:016X}\n"); idt.write_entry(&idte, 0x15);
        //Write IDTR
        //idt.write_idtr();
    }

    // PREBOOT SEQUENCE
    writeln!(printer, "\n=== PREBOOT ===\n");
    
    let entry_point_virtual = oct4_to_pointer(KERNEL_OCT).unwrap() as u64 + kernel.header.entry_point;
    let stack_point_virtual = oct4_to_pointer(KERNEL_OCT).unwrap() as u64 + PAGE_SIZE_512G as u64 - 1024;
    let pml4_point_physical = pml4.physical.0;
    writeln!(printer, "Entry Point Virtual:  {:16X}", entry_point_virtual);
    writeln!(printer, "Stack Point Virtual:  {:16X}", stack_point_virtual);
    writeln!(printer, "PML4  Point Physical: {:16X}", pml4_point_physical);
    //Push free memory to new page map
    {
        //Check amount of free memory
        let free_memory_types = &[MemoryType::CONVENTIONAL, MemoryType::BOOT_SERVICES_CODE, MemoryType::BOOT_SERVICES_DATA];
        let free_page_count: usize = uefi_free_memory_page_count(boot_services, free_memory_types).unwrap();
        writeln!(printer, "Free Memory Before Final Boot:      {:10}Pg or {:4}MiB {:4}KiB", free_page_count, (free_page_count*PAGE_SIZE_4KIB)/MIB, ((free_page_count*PAGE_SIZE_4KIB) % MIB)/KIB);
        
        //Check memory required to map free memory
        let required_page_count: usize = (free_page_count/PAGE_NUMBER_1) + 1;
        writeln!(printer, "Required Memory to Map Free Memory: {:10}Pg or {:4}MiB {:4}KiB", required_page_count, (required_page_count*PAGE_SIZE_4KIB)/MIB, ((required_page_count*PAGE_SIZE_4KIB) % MIB)/KIB);
        //Allocate and zero memory required
        let free_memory_stack_location = unsafe {allocate_memory(boot_services, MemoryType::LOADER_DATA, required_page_count * PAGE_SIZE_4KIB, PAGE_SIZE_4KIB)} as *mut u64;
        writeln!(printer, "Free Memory Stack Location: {:p}", free_memory_stack_location);
        unsafe {
            for i in 0..required_page_count*512 {
                write_volatile(free_memory_stack_location.add(i), 0);
            }
        }
        pml3_free_memory.map_pages_offset_4kib(PhysicalAddress(free_memory_stack_location as usize), 0, required_page_count * PAGE_SIZE_4KIB, true, false, true);
        
        //Retrieve uefi memory map
        let mut buffer = [0u8; 0x4000];
        let (_k, description_iterator) = match boot_services.memory_map(&mut buffer) {
            Ok(value) => value.unwrap(),
            Err(error) => {panic!("{}", uefi_error_readout(error.status()))}
        };
        //Iterate over memory map
        let mut running_total: u64 = 0;
        for descriptor in description_iterator {
            if free_memory_types.contains(&descriptor.ty) {
                let map_location = descriptor.phys_start;
                let map_pages = descriptor.page_count as usize;
                //writeln!(printer, "Free Memory:\n  Location: {:016X}\n  Pages:    {:016X}", map_location, map_pages);
                for i in 0..map_pages {
                    unsafe {write_volatile(free_memory_stack_location.add(running_total as usize + 1), map_location + (i * PAGE_SIZE_4KIB) as u64);}
                    running_total += 1;
                }
            }
        }
        //Finalize
        unsafe {write_volatile(free_memory_stack_location, running_total);}
        writeln!(printer, "Free Memory After Boot:             {:10}Pg or {:4}MiB {:4}KiB", running_total, (running_total as usize *PAGE_SIZE_4KIB)/MIB, ((running_total as usize *PAGE_SIZE_4KIB) % MIB)/KIB);
    }
    //Change CR3
    writeln!(printer, "Changing Memory Map.");
    unsafe {asm!(
        "MOV CR3, {pml4_pointer}",
        pml4_pointer = in(reg) pml4_point_physical,
    )}

    // DEBUGGING
    unsafe {
        write_volatile(0x1000 as *mut u8, 0xCD);
        write_volatile(0x1001 as *mut u8, 0x15);
    }

    // COMMAND LINE
    writeln!(printer, "\n=== COMMAND LINE READY ===");
    //Enter Read-Evaluate-Print Loop
    let mut leave: bool = true;
    let mut countdown = 5_000_000;
    loop {
        //Wait for key to be pressed
        match input.read_key().unwrap().unwrap() {
            Some(input_key) => {
                leave = false;
                //Check input key
                match input_key {
                    //Printable Key
                    Key::Printable(input_ucs16) => {
                        //Convert to usable type
                        let input_char = char::from(input_ucs16);
                        //User has hit enter
                        if input_char == '\r' {
                            //Return to bottom of screen
                            printer.end_down();
                            //Execute command and reset input stack
                            let mut buffer = [0u8; INPUT_LENGTH*4];
                            match inputter.to_str(&mut buffer) {
                                Ok(command) => {
                                    let return_code = command_processor(&mut printer, &system_table_boot, command);
                                    //Check return code
                                    match return_code {
                                        0x00 => { //Continue
                                        },
                                        0x01 => { //Boot
                                            break;
                                        },
                                        0x02 => { //Shutdown
                                            system_table_boot.boot_services().stall(5_000_000);
                                            system_table_boot.runtime_services().reset(ResetType::Shutdown, Status(0), None);
                                        },
                                        0x03 => { //Panic
                                            system_table_boot.boot_services().stall(5_000_000);
                                            panic!("Manually called panic.");
                                        },
                                        _    => { //Unrecognized
                                            panic!("Unexpected return code.");
                                        },
                                    }
                                }
                                Err(error) => {
                                    writeln!(printer, "{}", error);
                                }
                            }
                            inputter.flush(whitespace);
                        }
                        //User has typed a character
                        else {
                            inputter.push_render(CharacterTwoTone::<ColorBGRX>{codepoint: input_char, foreground: COLOR_BGRX_WHITE, background: COLOR_BGRX_BLACK}, whitespace);
                        }
                    }
                    //Modifier or Control Key
                    Key::Special(scancode) => {
                        match scancode {
                            ScanCode::HOME      => {printer.end_up();},
                            ScanCode::END       => {printer.end_down();},
                            ScanCode::UP        => {printer.line_up();},
                            ScanCode::DOWN      => {printer.line_down();},
                            ScanCode::PAGE_UP   => {printer.page_up();},
                            ScanCode::PAGE_DOWN => {printer.page_down();},
                            _ => {},
                        }
                    }
                }
            }
            None => {
                boot_services.stall(10_000);
                if leave {
                    countdown -= 10_000;
                    if countdown <= 0 {break}
                }
            },
        }
    }
    writeln!(printer, "Bootloader Command Line Exited.");

    // BOOT SEQUENCE
    //Exit Boot Services
    let mut memory_map_buffer = [0; 10000];
    let (_system_table_runtime, _esi) = system_table_boot.exit_boot_services(handle, &mut memory_map_buffer).expect_success("Boot services exit failed");
    uefi::alloc::exit_boot_services();
    writeln!(printer, "Boot Services exited.");
    //Enter Kernel
    writeln!(printer, "Entering kernel.");
    unsafe {asm!(
        "MOV RSP, {stack_pointer}",
        "CALL {entry_pointer}",
        stack_pointer = in(reg) stack_point_virtual,
        entry_pointer = in(reg) entry_point_virtual,
        options(nostack),
    )}
    //Halt Computer
    writeln!(printer, "Halt reached.");
    unsafe {asm!("HLT");}
    Status::SUCCESS
}


// PANIC HANDLER
//Panic Variables
static mut PANIC_WRITE_POINTER: Option<*mut dyn Write> = None;

//Panic Handler
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    unsafe {
        let printer = &mut *PANIC_WRITE_POINTER.unwrap();
        writeln!(printer, "{}", panic_info);
        command_crread(printer, &mut ALL.split(" "));
        asm!("HLT");
        loop {}
    }
}

static mut ALL: &str = "-all";

// UEFI FUNCTIONS
//Read a UEFI error status as a string
fn uefi_error_readout(error: Status) -> &'static str {
    match error {
        Status::SUCCESS               => "UEFI Success.",
        Status::WARN_UNKNOWN_GLYPH    => "UEFI Warning: Unknown Glyph.",
        Status::WARN_DELETE_FAILURE   => "UEFI Warning: File Delete Failure.",
        Status::WARN_WRITE_FAILURE    => "UEFI Warning: Handle Write Failure.",
        Status::WARN_BUFFER_TOO_SMALL => "UEFI Warning: Buffer Too Small, Data Truncated.",
        Status::WARN_STALE_DATA       => "UEFI Warning: Stale Data",
        Status::WARN_FILE_SYSTEM      => "UEFI Warning: Buffer Contains File System.",
        Status::WARN_RESET_REQUIRED   => "UEFI Warning: Reset Required.",
        Status::LOAD_ERROR            => "UEFI Error: Image Load Error.",
        Status::INVALID_PARAMETER     => "UEFI Error: Invalid Parameter Provided.",
        Status::UNSUPPORTED           => "UEFI Error: Unsupported Operation.",
        Status::BAD_BUFFER_SIZE       => "UEFI Error: Bad Buffer Size.",
        Status::BUFFER_TOO_SMALL      => "UEFI Error: Buffer Too Small.",
        Status::NOT_READY             => "UEFI Error: Not Ready.",
        Status::DEVICE_ERROR          => "UEFI Error: Physical Device Error",
        Status::WRITE_PROTECTED       => "UEFI Error: Device Write Protected.",
        Status::OUT_OF_RESOURCES      => "UEFI Error: Out of Resources.",
        Status::VOLUME_CORRUPTED      => "UEFI Error: Volume Corrupted.",
        Status::VOLUME_FULL           => "UEFI Error: Volume Full.",
        Status::NO_MEDIA              => "UEFI Error: Media Missing.",
        Status::MEDIA_CHANGED         => "UEFI Error: Media Changed.",
        Status::NOT_FOUND             => "UEFI Error: Item Not Found.",
        Status::ACCESS_DENIED         => "UEFI Error: Access Denied.",
        Status::NO_RESPONSE           => "UEFI Error: No Response.",
        Status::NO_MAPPING            => "UEFI Error: No Mapping.",
        Status::TIMEOUT               => "UEFI Error: Timeout.",
        Status::NOT_STARTED           => "UEFI Error: Protocol Not Started.",
        Status::ALREADY_STARTED       => "UEFI Error: Protocol Already Started.",
        Status::ABORTED               => "UEFI Error: Operation Aborted.",
        Status::ICMP_ERROR            => "UEFI Error: Network ICMP Error.",
        Status::TFTP_ERROR            => "UEFI Error: Network TFTP Error.",
        Status::PROTOCOL_ERROR        => "UEFI Error: Network Protocol Error.",
        Status::INCOMPATIBLE_VERSION  => "UEFI Error: Incompatible Version.",
        Status::SECURITY_VIOLATION    => "UEFI Error: Security Violation.",
        Status::CRC_ERROR             => "UEFI Error: Cyclic Redundancy Check Error.",
        Status::END_OF_MEDIA          => "UEFI Error: End of Media Reached.",
        Status::END_OF_FILE           => "UEFI Error: End of File Reached.",
        Status::INVALID_LANGUAGE      => "UEFI Error: Invalid Language.",
        Status::COMPROMISED_DATA      => "UEFI Error: Compromised Data.",
        Status::IP_ADDRESS_CONFLICT   => "UEFI Error: Network IP Address Conflict.",
        Status::HTTP_ERROR            => "UEFI Error: Network HTTP Error.",
        _                             => "UEFI Error: Error Unrecognized.",
    }
}

//Set a larger graphics mode
fn uefi_set_graphics_mode(gop: &mut GraphicsOutput) {
    let mode:Mode = gop.modes()
    .map(|mode| mode.expect("Graphics Output Protocol query of available modes failed.")).find(|mode| {
        let info = mode.info();
        info.resolution() == (SCREEN_WIDTH, SCREEN_HEIGHT) && info.stride() == SCREEN_WIDTH
    }).unwrap();
    gop.set_mode(&mode).expect_success("Graphics Output Protocol set mode failed.");
}

//Total size of free conventional memory
fn uefi_free_memory_page_count(boot_services: &BootServices, memory_types: &[MemoryType]) -> Result<usize, &'static str> {
    //Build a buffer big enough to handle the memory map
    let mut buffer = [0u8; 0x4000];
    //Read memory map into buffer
    let (_k, description_iterator) = match boot_services.memory_map(&mut buffer) {
        Ok(value) => value.unwrap(),
        Err(error) => {return Err(uefi_error_readout(error.status()));}
    };
    //Iterate over memory map
    let mut size_pages: usize = 0;
    for descriptor in description_iterator {
        if memory_types.contains(&descriptor.ty) {
            size_pages += descriptor.page_count as usize;
        }
    }
    Ok(size_pages)
}


// COMMAND PROCESSOR
//Evaluate and execute a bootloader command and return a code
fn command_processor(printer: &mut dyn Write, system_table: &SystemTable<Boot>, command_str: &str) -> u8 {
    //Get necessary objects from system table
    let boot_services = system_table.boot_services();
    let runtime_services = system_table.runtime_services();
    //Print command
    writeln!(printer, "\n>{}", command_str);
    //Split command into iterator
    let mut args: Split<&str> = command_str.split(" ");
    let command: &str = match args.next(){
        Some(s) => s,
        None => {writeln!(printer, "Processor: No command entered."); return 0;}
    };
    //Assess command
    match command {
        command if command.eq_ignore_ascii_case("boot")     => {writeln!       (printer, "Processor: Boot sequence requested.");       0x01},
        command if command.eq_ignore_ascii_case("shutdown") => {writeln!       (printer, "Processor: Shutdown requested.");            0x02},
        command if command.eq_ignore_ascii_case("panic")    => {writeln!       (printer, "Processor: Panic requested.");               0x03},
        command if command.eq_ignore_ascii_case("time")     => {command_time   (printer, runtime_services, &mut args);                 0x00},
        command if command.eq_ignore_ascii_case("memmap")   => {command_memmap (printer, boot_services, &mut args);                    0x00},
        command if command.eq_ignore_ascii_case("memread")  => {command_memread(printer, &mut args);                                   0x00},
        command if command.eq_ignore_ascii_case("crread")   => {command_crread (printer, &mut args);                                   0x00},
        command                                             => {writeln!       (printer, "Processor: {} is not recognized.", command); 0x00},
    }
}

//Display the time
fn command_time(printer: &mut dyn Write, runtime_services: &RuntimeServices, args: &mut Split<&str>) {
    //Processing
    if let Some(s) = args.next() {
    if s.starts_with('-') {writeln!(printer, "Invalid flag: {}.\n{}",     s, HELP_TIME); return}
    else                  {writeln!(printer, "Invalid argument: {}.\n{}", s, HELP_TIME); return}};
    
    //Time Display
    let t = runtime_services.get_time();
    match t {
        Ok(t) => {let t = t.log(); writeln!(printer, "{}-{:02}-{:02} {:02}:{:02}:{:02} UTC", t.year(), t.month(), t.day(), t.hour(), t.minute(), t.second());},
        Err(error) => {writeln!(printer, "Failed to retrieve time:\n{}", uefi_error_readout(error.status()));}
    }
}

//Display memory map contents to console
fn command_memmap(printer: &mut dyn Write, boot_services: &BootServices, args: &mut Split<&str>) {
    //Processing
    if let Some(s) = args.next() {
        if s.starts_with('-') {writeln!(printer, "Invalid flag: {}.\n{}",     s, HELP_MEMMAP); return}
        else                  {writeln!(printer, "Invalid argument: {}.\n{}", s, HELP_MEMMAP); return}};
    //Estimated map size
    let map_size = boot_services.memory_map_size();
    writeln!(printer, "Map size: {}", map_size);
    //Build a buffer big enough to handle the memory map
    let mut buffer = [0u8;0x4000];
    writeln!(printer, "Buffer len: {}", buffer.len());
    //Read memory map into buffer
    let (_k, description_iterator) = match boot_services.memory_map(&mut buffer) {
        Ok(value) => value.unwrap(),
        Err(error) => {writeln!(printer, "{}", uefi_error_readout(error.status())); return;}
    };
    //Print memory map
    let mut i = 0;
    for descriptor in description_iterator{
        let size_pages = descriptor.page_count;
        let size = size_pages * PAGE_SIZE_4KIB as u64;
        let end_address = descriptor.phys_start + size;
        let mut memory_type_text:&str =                              "RESERVED          ";
        match descriptor.ty {
            MemoryType::LOADER_CODE           => {memory_type_text = "LOADER CODE       "}
            MemoryType::LOADER_DATA           => {memory_type_text = "LOADER DATA       "}
            MemoryType::BOOT_SERVICES_CODE    => {memory_type_text = "BOOT CODE         "}
            MemoryType::BOOT_SERVICES_DATA    => {memory_type_text = "BOOT DATA         "}
            MemoryType::RUNTIME_SERVICES_CODE => {memory_type_text = "RUNTIME CODE      "}
            MemoryType::RUNTIME_SERVICES_DATA => {memory_type_text = "RUNTIME DATA      "}
            MemoryType::CONVENTIONAL          => {memory_type_text = "CONVENTIONAL      "}
            MemoryType::UNUSABLE              => {memory_type_text = "UNUSABLE          "}
            MemoryType::ACPI_RECLAIM          => {memory_type_text = "ACPI RECLAIM      "}
            MemoryType::ACPI_NON_VOLATILE     => {memory_type_text = "ACPI NONVOLATILE  "}
            MemoryType::MMIO                  => {memory_type_text = "MEMORY MAPPED IO  "}
            MemoryType::MMIO_PORT_SPACE       => {memory_type_text = "MEMORY MAPPED PORT"}
            MemoryType::PAL_CODE              => {memory_type_text = "PROCESSOR MEMORY  "}
            MemoryType::PERSISTENT_MEMORY     => {memory_type_text = "PERSISTENT        "}
            _ => {}
        }
        writeln!(printer, "{}: {:016x}-{:016x} ({:8}KiB/{:8}Pg)", memory_type_text, descriptor.phys_start, end_address, size/1024, size_pages);
        i += 1;
    }
    writeln!(printer, "Total entries: {}", i);
}

//Display the raw contents of a part of memory
fn command_memread(printer: &mut dyn Write, args: &mut Split<&str>) {
    //Pre processing variables
    #[derive(Debug)]
    #[derive(PartialEq)]
    enum Flag  {Memread, W, R, C, E, None}
    enum R {B, O, D, X}
    enum E {Big, Little}
    let mut flag = Flag::Memread;
    let mut memread: (usize, *mut u8) = (1, core::ptr::null_mut::<u8>());
    let mut flag_w:  (usize, usize)   = (1, 1);
    let mut flag_r:  (usize, R)       = (1, R::X);
    let mut flag_c:  (usize, usize)   = (1, 1);
    let mut flag_e:  (usize, E)       = (1, E::Little);
    
    //Processing
    for arg in args {
        if arg.starts_with('-') {match arg {
            arg if arg.eq_ignore_ascii_case("-w")  => {if !(flag == Flag::None) {writeln!(printer, "Flag {:?} must take 1 arguments.\n{}", flag, HELP_MEMREAD); return;} else if flag_w.0 == 1 {flag = Flag::W} else {writeln!(printer, "Flag w cannot be called more than once");}},
            arg if arg.eq_ignore_ascii_case("-r")  => {if !(flag == Flag::None) {writeln!(printer, "Flag {:?} must take 1 arguments.\n{}", flag, HELP_MEMREAD); return;} else if flag_r.0 == 1 {flag = Flag::R} else {writeln!(printer, "Flag r cannot be called more than once");}},
            arg if arg.eq_ignore_ascii_case("-c")  => {if !(flag == Flag::None) {writeln!(printer, "Flag {:?} must take 1 arguments.\n{}", flag, HELP_MEMREAD); return;} else if flag_c.0 == 1 {flag = Flag::C} else {writeln!(printer, "Flag c cannot be called more than once");}},
            arg if arg.eq_ignore_ascii_case("-e")  => {if !(flag == Flag::None) {writeln!(printer, "Flag {:?} must take 1 arguments.\n{}", flag, HELP_MEMREAD); return;} else if flag_e.0 == 1 {flag = Flag::E} else {writeln!(printer, "Flag e cannot be called more than once");}},
            _ => {writeln!(printer, "Invalid flag: {}.\n{}", arg, HELP_MEMREAD); return;}
        }}
        else {match flag{
            Flag::Memread => {
                match memread.0 {
                    1 => {
                        memread.1 = match usize::from_str_radix(arg, 16) {
                            Ok(a) => a, 
                            Err(_) => {writeln!(printer, "Could not parse number from argument: {}.\n{}", arg, HELP_MEMREAD); return;}
                        } as *mut u8;
                        memread.0 += 1;
                        flag = Flag::None;
                    }
                    _ => {writeln!(printer, "Invalid argument: {}.\n{}", arg, HELP_MEMREAD); return;}
                }
            },
            Flag::W => {
                match flag_w.0 {
                    1 => {
                        flag_w.1 = match arg {
                            arg if arg.eq_ignore_ascii_case("8")  => 1,
                            arg if arg.eq_ignore_ascii_case("16") => 2,
                            arg if arg.eq_ignore_ascii_case("24") => 3,
                            arg if arg.eq_ignore_ascii_case("32") => 4,
                            arg if arg.eq_ignore_ascii_case("48") => 6,
                            arg if arg.eq_ignore_ascii_case("64") => 8,
                            _ => {writeln!(printer, "Could not parse width from argument to -w: {}.\n{}", arg, HELP_MEMREAD); return;}
                        };
                        flag_w.0 += 1;
                        flag = Flag::None;
                    }
                    _ => {writeln!(printer, "Invalid argument to -w: {}.\n{}", arg, HELP_MEMREAD); return;}
                }
            },
            Flag::R => {
                match flag_r.0 {
                    1 => {
                        flag_r.1 = match arg {
                            arg if arg.eq_ignore_ascii_case("b") => R::B,
                            arg if arg.eq_ignore_ascii_case("o") => R::O,
                            arg if arg.eq_ignore_ascii_case("d") => R::D,
                            arg if arg.eq_ignore_ascii_case("x") => R::X,
                            _ => {writeln!(printer, "Could not parse radix from argument 1 to flag r: {}.\n{}", arg, HELP_MEMREAD); return;}
                        };
                        flag_r.0 += 1;
                        flag = Flag::None;
                    }
                    _ => {writeln!(printer, "Flag r takes 1 arguments not {}.\n{}", flag_r.0, HELP_MEMREAD); return;}
                }
            },
            Flag::C => {
                match flag_c.0 {
                    1 => {
                        flag_c.1 = match arg.parse::<usize>() {
                            Ok(a) => a,
                            Err(_) => {writeln!(printer, "Could not parse number from argument 1 to flag c: {}.\n{}", arg, HELP_MEMREAD); return;}
                        };
                        flag_c.0 += 1;
                        flag = Flag::None;
                    }
                    _ => {writeln!(printer, "Flag c takes 1 arguments not {}.\n{}", flag_c.0, HELP_MEMREAD); return;}
                }
            },
            Flag::E => {
                match flag_e.0 {
                    1 => {
                        flag_e.1 = match arg {
                            arg if arg.eq_ignore_ascii_case("big")    => E::Big,
                            arg if arg.eq_ignore_ascii_case("little") => E::Little,
                            _ => {writeln!(printer, "Could not parse endianness from argument 1 to flag e: {}.\n{}", arg, HELP_MEMREAD); return;}
                        };
                        flag_e.0 += 1;
                        flag = Flag::None;
                    }
                    _ => {writeln!(printer, "Flag e takes 1 arguments not: {}.\n{}", arg, HELP_MEMREAD); return;}
                }
            },
            Flag::None => {writeln!(printer, "Invalid argument: {}.\n{}", arg, HELP_MEMREAD); return;}
        }}
    }
    //Check validity
    if memread.0 == 1 {writeln!(printer, "{}", HELP_MEMREAD); return;}
    //Display memory
    let display: fn(&mut dyn Write, *mut u8, u64) = match (flag_w.1, flag_r.1) {
        (1, R::B) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0b{:08b}",  address, number);},
        (1, R::O) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0o{:03o}",  address, number);},
        (1, R::D) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0d{:02}",   address, number);},
        (1, R::X) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0x{:02X}",  address, number);},
        (2, R::B) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0b{:016b}", address, number);},
        (2, R::O) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0o{:06o}",  address, number);},
        (2, R::D) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0d{:05}",   address, number);},
        (2, R::X) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0x{:04X}",  address, number);},
        (3, R::B) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0b{:024b}", address, number);},
        (3, R::O) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0o{:08o}",  address, number);},
        (3, R::D) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0d{:08}",   address, number);},
        (3, R::X) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0x{:06X}",  address, number);},
        (4, R::B) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0b{:032b}", address, number);},
        (4, R::O) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0o{:011o}", address, number);},
        (4, R::D) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0d{:010}",  address, number);},
        (4, R::X) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0x{:08X}",  address, number);},
        (6, R::B) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0b{:048b}", address, number);},
        (6, R::O) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0o{:016o}", address, number);},
        (6, R::D) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0d{:015}",  address, number);},
        (6, R::X) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0x{:012X}", address, number);},
        (8, R::B) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0b{:064b}", address, number);},
        (8, R::O) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0o{:022o}", address, number);},
        (8, R::D) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0d{:020}",  address, number);},
        (8, R::X) => |printer: &mut dyn Write, address: *mut u8, number: u64| {writeln!(printer, "{:p}: 0x{:016X}", address, number);},
        _ => {writeln!(printer, "Error: bit width processed incorrectly."); return;}
    };
    let convert = match flag_e.1 {E::Big => u64::from_be_bytes, E::Little => u64::from_le_bytes};
    let address = memread.1;
    let width = flag_w.1;
    let count = flag_c.1;

    unsafe {for i in 0..count {
        let mut bytes: [u8; 8] = [0u8; 8];
        for j in 0..width {
            let mut byte: u8;
            asm!("mov {0}, [{1}]", out(reg_byte) byte, in(reg) address.add(i*width + j), options(readonly, nostack));
            match flag_e.1 {
                E::Big    => {bytes[8-j] = byte;},
                E::Little => {bytes[j]   = byte;},
            }
        }
        display(printer, address.add(i*width), convert(bytes));
    }}
}

//Display contents of control registers
fn command_crread(printer: &mut dyn Write, args: &mut Split<&str>) {
    //Pre processing variables
    let mut cr0:  bool = false;
    let mut cr2:  bool = false;
    let mut cr3:  bool = false;
    let mut cr4:  bool = false;
    let mut efer: bool = false;
    let mut gdtr: bool = false;
    //Processing
    for arg in args {
        if arg.starts_with('-') { match arg {
            arg if arg.eq_ignore_ascii_case("-all")  => {if cr0||cr2||cr3||cr4||efer||gdtr {writeln!(printer, "Flag usage not valid.\n{}", HELP_CRREAD); return;} else {cr0 = true; cr2 = true; cr3 = true; cr4 = true; efer = true; gdtr = true;}},
            arg if arg.eq_ignore_ascii_case("-cr0")  => {if cr0                            {writeln!(printer, "Flag usage not valid.\n{}", HELP_CRREAD); return;} else {cr0 = true}},
            arg if arg.eq_ignore_ascii_case("-cr2")  => {if cr2                            {writeln!(printer, "Flag usage not valid.\n{}", HELP_CRREAD); return;} else {cr2 = true}},
            arg if arg.eq_ignore_ascii_case("-cr3")  => {if cr3                            {writeln!(printer, "Flag usage not valid.\n{}", HELP_CRREAD); return;} else {cr3 = true}},
            arg if arg.eq_ignore_ascii_case("-cr4")  => {if cr4                            {writeln!(printer, "Flag usage not valid.\n{}", HELP_CRREAD); return;} else {cr4 = true}},
            arg if arg.eq_ignore_ascii_case("-efer") => {if efer                           {writeln!(printer, "Flag usage not valid.\n{}", HELP_CRREAD); return;} else {efer = true}},
            arg if arg.eq_ignore_ascii_case("-gdtr") => {if gdtr                           {writeln!(printer, "Flag usage not valid.\n{}", HELP_CRREAD); return;} else {gdtr = true}},
            _ => {writeln!(printer, "Invalid flag: {}.\n{}",     arg, HELP_CRREAD); return;}
        }}
        else     {writeln!(printer, "Invalid argument: {}.\n{}", arg, HELP_CRREAD); return;}
    }
    //Check validity
    if !(cr0||cr2||cr3||cr4||efer||gdtr) {writeln!(printer, "{}", HELP_CRREAD); return;}
    //Control Register Display
    if cr0  {writeln!(printer, "Control Register 0:\n  Flags:   0x{:016X}", Cr0::read().bits());}
    if cr2  {
        let addr: u64;
        unsafe {asm!("MOV {}, CR2", out(reg) addr,)}
        writeln!(printer, "Control Register 2:\n  Address: 0x{:016X}", addr);
    }
    if cr3  {writeln!(printer, "Control Register 3:\n  Flags:   0b{:016X}\n  Address: 0x{:016X}", Cr3::read().1.bits(), Cr3::read().0.start_address());}
    if cr4  {writeln!(printer, "Control Register 4:\n  Flags:   0x{:016X}", Cr4::read().bits());}
    if efer {writeln!(printer, "Extended Feature Enable Register:\n  Flags:   0x{:016X}", Efer::read().bits());}
    if gdtr {
        let gdt = [0u8; 10];
        unsafe {asm!("SGDT [{}]", in(reg) &gdt, options(nostack));}
        let gdta = u64::from_le_bytes(gdt[2..10].try_into().unwrap());
        let gdtl = u16::from_le_bytes(gdt[0..2].try_into().unwrap());
        writeln!(printer, "Global Descriptor Table Register:\n  Address: 0x{:016X}\n  Limit:   0x{:04X}", gdta, gdtl);
    }
}


// MEMORY FUNCTIONS
//Allocate memory
unsafe fn allocate_memory(boot_services: &BootServices, memory_type: MemoryType, size: usize, align: usize) -> *mut u8 {
    if align > 8 {
        let pointer =
            if let Ok(pointer_from_services) = boot_services.allocate_pool(memory_type, size + align).warning_as_error() {
                pointer_from_services
            }
            else {
                return ptr::null_mut();
            };
        let mut offset = pointer.align_offset(align);
        if offset == 0 {
            offset = align;
        }
        let pointer_return = pointer.add(offset);
        (pointer_return as *mut *mut u8).sub(1).write(pointer);
        pointer_return
    }
    else {
        boot_services.allocate_pool(memory_type, size).warning_as_error().unwrap_or(ptr::null_mut())
    }
}

//Allocate memory which has been zeroed
unsafe fn allocate_page_zeroed(boot_services: &BootServices, memory_type: MemoryType) -> PhysicalAddress {
    let pointer = allocate_memory(boot_services, memory_type, PAGE_SIZE_4KIB, PAGE_SIZE_4KIB);
    for i in 0..PAGE_SIZE_4KIB{
        write_volatile(pointer.add(i), 0x00);
    }
    PhysicalAddress(pointer as usize)
}


// STRUCTS
//File Read System
struct LocationalReadWrapper<'a> {
    ref_cell: RefCell<&'a mut RegularFile>,
}
impl<'a> LocationalRead for LocationalReadWrapper<'a> {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
        match self.ref_cell.try_borrow_mut() {
            Ok(mut file) => {
                file.set_position(offset as u64);
                match file.read(buffer) {
                    Ok(completion) => {
                        let size = completion.unwrap(); 
                        if size == buffer.len() {
                            Ok(())
                        } 
                        else {
                            Err("UEFI File Read: Buffer exceeds end of file.")
                        }
                    },
                    Err(error) => Err(uefi_error_readout(error.status())),
                }
            },
            Err(error) => {panic!("{}", error)}
        }
    }
}

//UEFI Page Allocator
struct UefiPageAllocator<'a> {
    boot_services: &'a BootServices,
}
impl<'a> PageAllocator for UefiPageAllocator<'a> {
    fn allocate_page     (&self)                            -> Result<PhysicalAddress, &'static str> {
        Ok(unsafe {allocate_page_zeroed(self.boot_services, MemoryType::LOADER_DATA)})
    }
    fn deallocate_page   (&self, _physical: PhysicalAddress) -> Result<(),              &'static str> {
        Ok(())
    }
    fn physical_to_linear(&self, physical: PhysicalAddress) -> Result<LinearAddress,   &'static str> {
        Ok(LinearAddress(physical.0))
    }
}


// STRINGS
//Help String for time
const HELP_TIME: &str = "\
TIME        : Display the computer's Unix time in UTC.
time        : Command takes no arguments.";

//Help String for memmap
const HELP_MEMMAP: &str = "\
MEMMAP      : Display the contents of the UEFI memory map.
memmap      : Command takes no arguments.";

//Help String for memread
const HELP_MEMREAD: &str = "\
MEMREAD     : Display an aribtrary portion of memory.
memread     : [address] (-w [width]) (-r [radix]) (-c [count]) (-e [endianness])
address     : Hexidecimal number: address to begin reading from.
-w          : Read using and display with the given bit [width].
 width      : '8', '16', '24', '32', '48', or '64'. Default of '8'.
-r          : Display with the given [radix].
 radix      : 'b'inary, 'o'ctal, 'd'ecimal, or he'x'idecimal. Default of 'x'.
-c          : Sequentially repeat the read with the given [count].
 count      : Hexidecimal number: amount of reads to make. Default of 1.
-e          : Read using and display with the given [endianness].
 endianness : 'big' or 'little'. Default of 'little'.";

//Help String for crread
const HELP_CRREAD: &str = "\
CRREAD      : Display the x86-64 control registers.
crread      : [-all | -cr0 -cr2 -cr3 -cr4 -efer]
-all        : Display All Registers
-cr0        : Display Control Register 0
-cr2        : Display Control Register 2
-cr3        : Display Control Register 3
-cr4        : Display Control Register 4
-efer       : Display Extended Feature Enable Register";
