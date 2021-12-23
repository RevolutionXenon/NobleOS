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
#![feature(asm)]
#![feature(bench_black_box)]
#![feature(box_syntax)]

//External crates
extern crate rlibc;
extern crate alloc;

//Imports
use photon::*;
use gluon::*;
use gluon::sysv_executable::*;
use gluon::x86_64_paging::*;
use core::cell::Cell;
use core::cell::RefCell;
use core::convert::TryInto;
use core::fmt::Write;
#[cfg(not(test))]
use core::panic::PanicInfo;
use core::ptr;
use core::ptr::write_volatile;
use core::str::Split;
use uefi::alloc::exit_boot_services;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::gop::Mode;
use uefi::proto::console::text::Input;
use uefi::proto::console::text::Key;
use uefi::proto::console::text::ScanCode;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileMode;
use uefi::proto::media::file::RegularFile;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::MemoryType;
use uefi::table::runtime::ResetType;
use x86_64::registers::control::*;

//Constants
const HYDROGEN_VERSION: &str = "vDEV-2021-12-13"; //CURRENT VERSION OF BOOTLOADER


// MAIN
//Main Entry Point After UEFI Boot
#[entry]
fn efi_main(_handle: Handle, system_table_boot: SystemTable<Boot>) -> Status {
    // UEFI INITILIZATION
    //Utilities Initialization (Alloc)
    uefi_services::init(&system_table_boot).expect_success("UEFI Initialization: failed at utilities initialization.");
    let boot_services = system_table_boot.boot_services();
    //Console Reset
    system_table_boot.stdout().reset(false).expect_success("Console reset failed.");
    //Watchdog Timer Shutoff
    boot_services.set_watchdog_timer(0, 0x10000, Some(&mut {&mut [0x0058u16, 0x0000u16]}[..])).expect_success("UEFI Initialization: Watchdog Timer shutoff failed.");
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
    let pixel_renderer: PixelRendererHWD<ColorBGRX> = PixelRendererHWD {pointer: graphics_frame_pointer as *mut ColorBGRX, height: F1_SCREEN_HEIGHT, width: F1_SCREEN_WIDTH};
    let character_renderer: CharacterTwoToneRenderer16x16<ColorBGRX> = CharacterTwoToneRenderer16x16::<ColorBGRX> {renderer: &pixel_renderer, height: F1_FRAME_HEIGHT, width: F1_FRAME_WIDTH, y: 0, x: 0};
    let mut frame: FrameWindow::<F1_FRAME_HEIGHT, F1_FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = FrameWindow::<F1_FRAME_HEIGHT, F1_FRAME_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, whitespace, 0, 0);
    let mut printer: PrintWindow::<F1_PRINT_LINES, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>> = PrintWindow::<F1_PRINT_LINES, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, whitespace, whitespace, F1_PRINT_Y, F1_PRINT_X);
    let mut inputter: InputWindow::<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>  = InputWindow::<F1_INPUT_LENGTH, F1_INPUT_WIDTH, ColorBGRX, CharacterTwoTone<ColorBGRX>>::new(&character_renderer, whitespace, F1_INPUT_Y, F1_INPUT_X);
    unsafe {PANIC_WRITE_POINTER = Some(&mut printer as &mut dyn Write as *mut dyn Write)};
    //Graphics Output Protocol: Set Graphics Mode
    uefi_set_graphics_mode(graphics_output_protocol);
    let _st = graphics_output_protocol.current_mode_info().stride();
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
    frame.horizontal_line(F1_PRINT_Y-1,     0,            F1_FRAME_WIDTH-1,  bluespace);
    frame.horizontal_line(F1_INPUT_Y-1,     0,            F1_FRAME_WIDTH-1,  bluespace);
    frame.horizontal_line(F1_INPUT_Y+1,     0,            F1_FRAME_WIDTH-1,  bluespace);
    frame.vertical_line(  0,                F1_PRINT_Y-1, F1_INPUT_Y+1,      bluespace);
    frame.vertical_line(  F1_FRAME_WIDTH-1, F1_PRINT_Y-1, F1_INPUT_Y+1,      bluespace);
    frame.horizontal_string("NOBLE OS",            0, 0,                                            bluespace);
    frame.horizontal_string("HYDROGEN BOOTLOADER", 0, F1_FRAME_WIDTH - 20 - HYDROGEN_VERSION.len(), bluespace);
    frame.horizontal_string(HYDROGEN_VERSION,      0, F1_FRAME_WIDTH -      HYDROGEN_VERSION.len(), bluespace);
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

    // BOOT LOAD
    let pml4: PageMap;
    let page_allocator = UefiPageAllocator{boot_services};
    unsafe {
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
            pml3_frame_buffer         .map_pages_offset_4kib(PhysicalAddress(graphics_frame_pointer as usize), 0, F1_SCREEN_HEIGHT*F1_SCREEN_WIDTH*4, true, true, true).unwrap();
        writeln!(printer, "PML3 FRAME: 0x{:16X}", pml3_frame_buffer.linear.0);
        //Page Map Level 3: Offset Identity Map
        let pml3_offset_identity_map: PageMap = PageMap::new(allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA), PageMapLevel::L3, &page_allocator).unwrap();
            pml3_offset_identity_map  .map_pages_offset_1gib(PhysicalAddress(0), 0, PAGE_SIZE_512G, true, true, true).unwrap();
        writeln!(printer, "PML3 IDENT: 0x{:16X}", pml3_offset_identity_map.linear.0);
        //Write PML4 Entries
        pml4.write_entry(PHYSICAL_OCT,     PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_efi_physical_memory.physical, true, true, false).unwrap()).unwrap();
        pml4.write_entry(KERNEL_OCT,       PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_kernel             .physical, true, true, false).unwrap()).unwrap();
        pml4.write_entry(STACKS_OCT,       PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_kernel_stacks      .physical, true, false,false).unwrap()).unwrap();
        pml4.write_entry(FRAME_BUFFER_OCT, PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_frame_buffer       .physical, true, true, true ).unwrap()).unwrap();
        pml4.write_entry(IDENTITY_OCT,     PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_offset_identity_map.physical, true, true, true ).unwrap()).unwrap();
        pml4.write_entry(PAGE_MAP_OCT,     PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml4                    .physical, true, true, true ).unwrap()).unwrap();
        // FINISH
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
                            let mut buffer = [0u8; F1_INPUT_LENGTH*4];
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

    // BOOT SEQUENCE
    writeln!(printer, "\n=== BOOTING ===\n");
    writeln!(printer, "Bootloader Command Line Exited.");
    let entry_point_virtual = oct4_to_pointer(KERNEL_OCT).unwrap() as u64 + kernel.header.entry_point;
    let stack_point_virtual = oct4_to_pointer(KERNEL_OCT).unwrap() as u64 + PAGE_SIZE_512G as u64;
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
        let required_pages_l1: usize = (free_page_count + PAGE_NUMBER_1 - 1) / PAGE_NUMBER_1;
        let required_pages_l2: usize = (free_page_count + PAGE_NUMBER_2 - 1) / PAGE_NUMBER_2;
        let required_pages_l3: usize = (free_page_count + PAGE_NUMBER_3 - 1) / PAGE_NUMBER_3;
        let required_page_count: usize = required_pages_l1 + required_pages_l2 + required_pages_l3;
        writeln!(printer, "Required Memory to Map Free Memory: {:10}Pg or {:4}MiB {:4}KiB", required_page_count, (required_page_count*PAGE_SIZE_4KIB)/MIB, ((required_page_count*PAGE_SIZE_4KIB) % MIB)/KIB);
        //Allocate and zero memory required
        let zone_location = unsafe {allocate_memory(boot_services, MemoryType::LOADER_DATA, required_page_count * PAGE_SIZE_4KIB, PAGE_SIZE_4KIB)};
        unsafe {
            for i in 0..required_page_count*PAGE_SIZE_4KIB {
                write_volatile(zone_location.add(i), 0x00);
            }
        }
        let zone_page_allocator = ZonePageAllocator::new(zone_location, required_page_count).unwrap();
        //Build page map
        let pml3_free_memory = PageMap::new(zone_page_allocator.allocate_page().unwrap(), PageMapLevel::L3, &zone_page_allocator).unwrap();
        let mut buffer = [0u8; 0x4000];
        let (_k, description_iterator) = match boot_services.memory_map(&mut buffer) {
            Ok(value) => value.unwrap(),
            Err(error) => {panic!("{}", uefi_error_readout(error.status()))}
        };
        //Iterate over memory map
        let mut size_pages = 0;
        for descriptor in description_iterator {
            let free_memory_address = PhysicalAddress(descriptor.phys_start as usize);
            let free_memory_size = (descriptor.page_count as usize) * PAGE_SIZE_4KIB;
            if free_memory_types.contains(&descriptor.ty) {
                //writeln!(printer, "Free Memory Location:     {:16X} ({:8}KiB, {:8}pg)", free_memory_pointer as usize, free_memory_size/KIB, free_memory_size/PAGE_SIZE_4KIB);
                pml3_free_memory.map_pages_offset_4kib(free_memory_address, size_pages*PAGE_SIZE_4KIB, free_memory_size, true, true, false).unwrap();
                size_pages += descriptor.page_count as usize;
            }
        }
        writeln!(printer, "Free Memory After Final Boot:       {:10}Pg or {:4}MiB {:4}KiB", size_pages, (size_pages*PAGE_SIZE_4KIB)/MIB, ((size_pages*PAGE_SIZE_4KIB) % MIB)/KIB);
        //Write into pml4
        let pml4e_free_memory = PageMapEntry::new(PageMapLevel::L4, PageMapEntryType::Table, pml3_free_memory.physical, true, true, false).unwrap();
        pml4.write_entry(FREE_MEMORY_OCT, pml4e_free_memory).unwrap();
    }
    //Delay
    //writeln!(printer, "Holding for 5 seconds.");
    //boot_services.stall(5_000_000);
    //Exit Boot Services
    let mut memory_map_buffer = [0; 10000];
    let (_table_runtime, _esi) = system_table_boot.exit_boot_services(_handle, &mut memory_map_buffer).expect_success("Boot services exit failed");
    exit_boot_services();
    writeln!(printer, "Boot Services exited.");
    //Enter Kernel
    unsafe {
        asm!("MOV R14, {stack}",   stack   = in(reg) stack_point_virtual, options(nostack));
        asm!("MOV R15, {entry}",   entry   = in(reg) entry_point_virtual, options(nostack));
        asm!("MOV CR3, {pagemap}", pagemap = in(reg) pml4_point_physical, options(nostack));
        asm!("MOV RSP, R14",                                              options(nostack));
        asm!("JMP R15",                                                   options(nostack));
    }
    //Halt Computer
    writeln!(printer, "Halt reached.");
    unsafe {asm!("HLT");}
    Status::SUCCESS
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
        info.resolution() == (F1_SCREEN_WIDTH, F1_SCREEN_HEIGHT)
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
        let mut memory_type_text:&str =                              "RESERVED             ";
        match descriptor.ty {
            MemoryType::LOADER_CODE           => {memory_type_text = "LOADER CODE          "}
            MemoryType::LOADER_DATA           => {memory_type_text = "LOADER DATA          "}
            MemoryType::BOOT_SERVICES_CODE    => {memory_type_text = "BOOT SERVICES CODE   "}
            MemoryType::BOOT_SERVICES_DATA    => {memory_type_text = "BOOT SERVICES DATA   "}
            MemoryType::RUNTIME_SERVICES_CODE => {memory_type_text = "RUNTIME SERVICES CODE"}
            MemoryType::RUNTIME_SERVICES_DATA => {memory_type_text = "RUNTIME SERVICES DATA"}
            MemoryType::CONVENTIONAL          => {memory_type_text = "CONVENTIONAL         "}
            MemoryType::UNUSABLE              => {memory_type_text = "UNUSABLE             "}
            MemoryType::ACPI_RECLAIM          => {memory_type_text = "ACPI RECLAIM         "}
            MemoryType::ACPI_NON_VOLATILE     => {memory_type_text = "ACPI NON VOLATILE    "}
            MemoryType::MMIO                  => {memory_type_text = "MEMORY MAPPED IO     "}
            MemoryType::MMIO_PORT_SPACE       => {memory_type_text = "MEMORY MAPPED PORT   "}
            MemoryType::PAL_CODE              => {memory_type_text = "PROCESSOR MEMORY     "}
            MemoryType::PERSISTENT_MEMORY     => {memory_type_text = "PERSISTENT           "}
            _ => {}
        }
        writeln!(printer, "{}: {:016x}-{:016x} ({:8}KiB / {:8}Pg)", memory_type_text, descriptor.phys_start, end_address, size/1024, size_pages);
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
    if cr2  {writeln!(printer, "Control Register 2:\n  Address: 0x{:016X}", Cr2::read().as_u64());}
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

//Zone Page Allocator
struct ZonePageAllocator {
    pub location: *mut u8,
    pub total_pages: usize,
    current_page: Cell<usize>,
}
impl ZonePageAllocator {
    pub fn new(location: *mut u8, total_pages: usize) -> Result<Self, &'static str> {
        if location as usize % PAGE_SIZE_4KIB != 0 {return Err("Zone Page Allocator: Location not aligned to 4KiB boundary.")}
        Ok(Self {
            location,
            total_pages,
            current_page: Cell::<usize>::new(0),
        })
    }
}
impl PageAllocator for ZonePageAllocator {
    fn allocate_page     (&self)                            -> Result<PhysicalAddress, &'static str> {
        if self.current_page.get() >= self.total_pages {
            Err("Zone Page Allocator: Out of pages.")
        }
        else {
            let location: *mut u8 = unsafe {self.location.add(self.current_page.get()*PAGE_SIZE_4KIB)};
            self.current_page.replace(self.current_page.get() + 1);
            Ok(PhysicalAddress(location as usize))
        }
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
