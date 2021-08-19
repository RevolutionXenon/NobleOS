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
#![feature(abi_efiapi)]
#![feature(box_syntax)]
#![feature(asm)]
#![feature(bench_black_box)]

//External crates
extern crate rlibc;
extern crate alloc;

use gluon::program::ProgramIterator;
use gluon::program_dynamic_entry::ProgramDynamicEntryIterator;
//Imports
use gluon::header::ApplicationBinaryInterface;
use gluon::header::InstructionSetArchitecture;
use gluon::header::ObjectType;
use gluon::program::ProgramType;
use photon::*;
use gluon::*;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr;
use core::ptr::read_volatile;
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
const HYDROGEN_VERSION: & str = "vDEV-2021-08-18"; //CURRENT VERSION OF BOOTLOADER


// MAIN
//Main Entry Point After UEFI Boot
#[entry]
fn efi_main(_handle: Handle, system_table_boot: SystemTable<Boot>) -> Status {
    // UEFI INITILIZATION
    //Utilities initialization (Alloc)
    uefi_services::init(&system_table_boot).expect_success("UEFI Initialization: Utilities initialization failed.");
    let boot_services = system_table_boot.boot_services();
    //Console reset
    system_table_boot.stdout().reset(false).expect_success("Console reset failed.");
    //Watchdog Timer shutoff
    boot_services.set_watchdog_timer(0, 0x10000, Some(&mut {&mut [0x0058u16, 0x0000u16]}[..])).expect_success("UEFI Initialization: Watchdog Timer shutoff failed.");
    //Graphics Output Protocol initialization
    let graphics_output_protocol = match boot_services.locate_protocol::<GraphicsOutput>() {
        Ok(gop) => gop,
        Err(_) => panic!("UEFI Initialization: Graphics Output Protocol not found.")
    };
    let graphics_output_protocol = graphics_output_protocol.expect("Graphics Output Protocol initialization failed at unsafe cell");
    let graphics_output_protocol = unsafe {&mut *graphics_output_protocol.get()};
    let graphics_frame_pointer = graphics_output_protocol.frame_buffer().as_mut_ptr();
    unsafe {PHYSICAL_FRAME_POINTER = graphics_frame_pointer};
    //Screen Variables
    let whitespace = Character::<BGRX_DEPTH>::new(' ', COLOR_WHT_BGRX, COLOR_BLK_BGRX);
    let bluespace  = Character::<BGRX_DEPTH>::new(' ', COLOR_BLU_BGRX, COLOR_BLK_BGRX);
    let renderer = Renderer::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH>::new(graphics_frame_pointer);
    let mut frame = CharacterFrame::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH, F1_FRAME_HEIGHT, F1_FRAME_WIDTH>::new(renderer, whitespace);
    let mut printer = PrintWindow::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH, F1_PRINT_LINES, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, F1_PRINT_Y, F1_PRINT_X>::new(renderer, whitespace, whitespace);
    let mut inputter = InputWindow::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH, F1_INPUT_LENGTH, F1_INPUT_WIDTH, F1_INPUT_Y, F1_INPUT_X>::new(renderer, whitespace);
    //Graphics Output Protocol set graphics mode
    set_graphics_mode(graphics_output_protocol);
    let _st = graphics_output_protocol.current_mode_info().stride();
    let _pf = graphics_output_protocol.current_mode_info().pixel_format();
    let _s = graphics_output_protocol.frame_buffer().size();
    //Simple File System initialization
    let simple_file_system = match boot_services.locate_protocol::<SimpleFileSystem>() {
        Ok(sfs) => sfs,
        Err(error) => panic!("{}", uefi_error_readout(error.status())),
    };
    let simple_file_system = simple_file_system.expect("Simjple File System initialization failed at unsafe cell.");
    let simple_file_system = unsafe {&mut *simple_file_system.get()};
    //Input initialization
    let input = match boot_services.locate_protocol::<Input>() {
        Ok(ink) => ink,
        Err(error) => panic!("{}", uefi_error_readout(error.status())),
    };
    let input = input.expect("Input initialization failed at unsafe cell.");
    let input = unsafe {&mut *input.get()};

    // FILE READ SYSTEM
    //Wrapper Struct
    struct FileWrapper<'a>{
        ref_cell: RefCell<&'a mut RegularFile>,
    }
    //Locational Read Implementation
    impl<'a> LocationalRead for FileWrapper<'a>{
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

    // GRAPHICS SETUP
    //User Interface initialization
    frame.horizontal_line(F1_PRINT_Y-1,     0,            F1_FRAME_WIDTH-1,  bluespace);
    frame.horizontal_line(F1_INPUT_Y-1,     0,            F1_FRAME_WIDTH-1,  bluespace);
    frame.horizontal_line(F1_INPUT_Y+1,     0,            F1_FRAME_WIDTH-1,  bluespace);
    frame.vertical_line(  0,                F1_PRINT_Y-1, F1_INPUT_Y+1,      bluespace);
    frame.vertical_line(  F1_FRAME_WIDTH-1, F1_PRINT_Y-1, F1_INPUT_Y+1,      bluespace);
    frame.horizontal_string("NOBLE OS",            0, 0,                                                     bluespace);
    frame.horizontal_string("HYDROGEN BOOTLOADER", 0, F1_FRAME_WIDTH - 20 - HYDROGEN_VERSION.len(), bluespace);
    frame.horizontal_string(HYDROGEN_VERSION,      0, F1_FRAME_WIDTH -      HYDROGEN_VERSION.len(), bluespace);
    frame.render();
    writeln!(printer, "\n=== WELCOME TO NOBLE OS ===\n");
    writeln!(printer, "Hydrogen Bootloader     {}", HYDROGEN_VERSION);
    writeln!(printer, "Photon Graphics Library {}", PHOTON_VERSION);
    writeln!(printer, "Gluon Memory Library    {}", GLUON_VERSION);

    // LOAD KERNEL
    //Find kernel on disk
    let mut sfs_dir_root = simple_file_system.open_volume().expect_success("File system root failed to open.");
    let sfs_kernel_handle = sfs_dir_root.open("noble", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"noble\".").
        open("helium", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"helium\".").
        open("x86-64.elf", FileMode::Read, FileAttribute::empty()).expect_success("File system kernel open failed at \"x86-64.elf\".");
    let mut sfs_kernel = unsafe {RegularFile::new(sfs_kernel_handle)};
    writeln!(printer, "Found kernel on file system.");
    //Read kernel file
    let mut sfs_kernel_wrap = FileWrapper{ref_cell: RefCell::new(&mut sfs_kernel)};
    let mut kernel = match ELFFile::new(&mut sfs_kernel_wrap) {
        Ok(elffile) =>  elffile,
        Err(error) => panic!("{}", error),
    };
    //Check ELF header validity
    if kernel.header.binary_interface         != ApplicationBinaryInterface::None     {writeln!(printer, "Kernel load: Incorrect Application Binary Interface (ei_osabi). Should be SystemV/None (0x00)."); panic!();}
    if kernel.header.binary_interface_version != 0x00                                 {writeln!(printer, "Kernel load: Incorrect Application Binary Interface Version (ei_abiversion). Should be None (0x00)."); panic!();}
    if kernel.header.architecture             != InstructionSetArchitecture::EmX86_64 {writeln!(printer, "Kernel load: Incorrect Instruction Set Architecture (e_machine). Should be x86-64 (0x3E)."); panic!();}
    if kernel.header.object_type              != ObjectType::Shared                   {writeln!(printer, "Kernel Load: Incorrect Object Type (e_type). Should be Dynamic (0x03)."); panic!()}
    //Print ELF header info
    {
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
        writeln!(printer, "Kernel Code Size:                   0x{:16X}", kernel.program_memory_size());
    }
    //Allocate memory for code
    let kernel_stack_size: usize = 16*MIB;
    let kernel_total_size: usize = kernel.program_memory_size() as usize + kernel_stack_size;
    let kernel_physical: *mut u8 = unsafe { allocate_memory(boot_services, MemoryType::LOADER_CODE, kernel_total_size, PAGE_SIZE_4KIB as usize) };
    //Load code into memory
    kernel.load(kernel_physical);
    //Check
    writeln!(printer, "\n=== PROGRAMS ===\n");
    let mut pi = 0;
    for program in kernel.programs() {
        match program {
            Ok(program) => writeln!(printer, "Program Found at Index {}: {:?}", pi, program),
            Err(error) => writeln!(printer, "Program Error at Index {}: {}", pi, error),
        };
        pi += 1;
    }
    writeln!(printer, "\n=== SECTIONS ===\n");
    let mut si = 0;
    for section in kernel.sections(){
        match section {
            Ok(section) => writeln!(printer, "Section Found at Index {}: {:?}", si, section),
            Err(error) => writeln!(printer, "Section Error at Index {}: {}", si, error)
        };
        si += 1;
    }
    //Relocation
    writeln!(printer, "\n=== DYNAMIC INFO ===\n");
    for program in ProgramIterator::new(kernel.file, &kernel.header) {
        match program {
            Ok(program) => {
                if program.program_type == ProgramType::Dynamic {
                    writeln!(printer, "Dynamic Table Found: {:#?}", program);
                    let dynamic_table = ProgramDynamicEntryIterator::new(kernel.file, &kernel.header, &program);
                    for dynamic_entry in dynamic_table {
                        match dynamic_entry {
                            Ok(entry) => {writeln!(printer, "Dynamic Table Entry: {:#?}", entry);},
                            Err(error) => {writeln!(printer, "Dynamic Table Error: {}", error);},
                        }
                    }
                }
            },
            Err(_) => {},
        }
    }

    // BOOT LOAD
    let pml4_knenv: *mut u8;
    unsafe {
        writeln!(printer, "=== PAGE TABLES ===");
        // PAGE TABLES
        //Page Map Level 4: Kernel Environment
        pml4_knenv = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
        writeln!(printer, "PML4 KNENV: 0o{0:016o} 0x{0:016X}", pml4_knenv as usize);
        //Page Map Level 4: EFI Boot
        let pml4_efibt:*mut u8 = Cr3::read().0.start_address().as_u64() as *mut u8;
        writeln!(printer, "PML4 EFIBT: 0o{0:016o} 0x{0:016X}", pml4_efibt as usize);
        //Page Map Level 3: EFI Boot Physical Memory
        let pml3_efiph:*mut u8 = read_pte(pml4_efibt, 0);
        writeln!(printer, "PML3 EFIPH: 0o{0:016o} 0x{0:016X}", pml3_efiph as usize);
        //Page Map Level 3: Operating System Initialized Physical Memory
        let pml3_osiph:*mut u8 = create_pml3_offset_1gib(boot_services, 0 as *mut u8, PAGE_SIZE_512G);
        writeln!(printer, "PML3 OSIPH: 0o{0:016o} 0x{0:016X}", pml3_osiph as usize);
        //Page Map Level 3: Kernel
        let pml3_kernl = create_pml3_offset_4kib(boot_services, kernel_physical, kernel_total_size);
        writeln!(printer, "PML3 KERNL: 0o{0:016o} 0x{0:016X}", pml3_kernl as usize);
        //Page Map Level 3: Frame Buffer
        let pml3_frame = create_pml3_offset_2mib(boot_services, graphics_frame_pointer, F1_SCREEN_HEIGHT*F1_SCREEN_WIDTH*BGRX_DEPTH);
        writeln!(printer, "PML3 FRAME: 0o{0:016o} 0x{0:016X}", pml3_frame as usize);
        //Write PML4 Entries
        write_pte(pml4_knenv, pml3_efiph, PHYSICAL_MEMORY_PHYSICAL_OCTAL, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        write_pte(pml4_knenv, pml3_kernl, KERNEL_VIRTUAL_OCTAL, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        write_pte(pml4_knenv, pml3_frame, FRAME_BUFFER_VIRTUAL_OCTAL, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        write_pte(pml4_knenv, pml3_osiph, PHYSICAL_MEMORY_VIRTUAL_OCTAL, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        write_pte(pml4_knenv, pml4_knenv, PAGE_MAP_VIRTUAL_OCTAL, PAGE_SIZE_4KIB, false, false, false, false, false, true);
    }

    // COMMAND LINE
    //Enter Read-Evaluate-Print Loop
    loop{
        //Wait for key to be pressed
        boot_services.wait_for_event(&mut [input.wait_for_key_event()]).expect_success("Boot services event wait failed");
        //Check input key
        let input_key = input.read_key().expect_success("Key input failed").unwrap();
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
                    let mut buffer = [' '; F1_INPUT_LENGTH];
                    let command = &inputter.to_chararray(&mut buffer).iter().collect::<String>();
                    let return_code = command_processor(&mut printer, &system_table_boot, command);
                    inputter.flush(whitespace);
                    //Check return code
                    match return_code{
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
                //User has typed a character
                else {
                    inputter.push_render(Character::new(input_char, COLOR_WHT_BGRX, COLOR_BLK_BGRX), whitespace);
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
    writeln!(printer, "\n=== BOOTING ===\n");
    writeln!(printer, "Bootloader Command Line Exited.");

    // BOOT SEQUENCE
    //Exit Boot Services
    let mut memory_map_buffer = [0; 10000];
    let (_table_runtime, _esi) = system_table_boot.exit_boot_services(_handle, &mut memory_map_buffer).expect_success("Boot services exit failed");
    exit_boot_services();
    writeln!(printer, "Boot Services exited.");

    //Enter Kernel
    unsafe{asm!(
        "MOV R14, {stack}",
        "MOV R15, {entry}",
        "MOV CR3, {pagemap}",
        "MOV RSP, R14",
        "JMP R15",

        stack = in(reg) KERNEL_VIRTUAL_POINTER as u64 + kernel_total_size as u64,
        entry = in(reg) KERNEL_VIRTUAL_POINTER as u64 + kernel.header.entry_point,
        pagemap = in(reg) pml4_knenv as u64,
        options(nostack)
    );}

    //Halt Computer
    writeln!(printer, "Halt reached.");
    unsafe {asm!("HLT");}
    loop {}
}


// PANIC HANDLER
//Panic Variables
static mut PHYSICAL_FRAME_POINTER: *mut u8 = 0 as *mut u8;

//Panic Handler
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    unsafe {
        if PHYSICAL_FRAME_POINTER != 0 as *mut u8 {
            let renderer = Renderer::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH>::new(PHYSICAL_FRAME_POINTER);
            let whitespace = Character::<BGRX_DEPTH>::new(' ', COLOR_WHT_BGRX, COLOR_BLK_BGRX);
            let blackspace = Character::<BGRX_DEPTH>::new(' ', COLOR_BLK_BGRX, COLOR_WHT_BGRX);
            let mut printer = PrintWindow::<F1_SCREEN_HEIGHT, F1_SCREEN_WIDTH, BGRX_DEPTH, F1_PRINT_HEIGHT, F1_PRINT_HEIGHT, F1_PRINT_WIDTH, F1_PRINT_Y, F1_PRINT_X>::new(renderer, blackspace, whitespace);
            printer.push_render("BOOTLOADER PANIC!\n", blackspace);
            writeln!(printer, "{}", panic_info);
        }
        asm!("HLT");
        loop {}
    }
}


// UEFI FUNCTIONS
//Read a UEFI error status as a string
fn uefi_error_readout(error: Status) -> &'static str{
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
fn set_graphics_mode(gop: &mut GraphicsOutput) {
    let mode:Mode = gop.modes().map(|mode| mode.expect("Graphics Output Protocol query of available modes failed.")).find(|mode| {
            let info = mode.info();
            info.resolution() == (F1_SCREEN_WIDTH, F1_SCREEN_HEIGHT)
        }).unwrap();
    gop.set_mode(&mode).expect_success("Graphics Output Protocol set mode failed.");
}


// COMMAND PROCESSOR
//Evaluate and execute a bootloader command and return a code
fn command_processor(printer: &mut dyn Write, system_table: &SystemTable<Boot>, command_str: &str) -> u8 {
    //Get necessary objects from system table
    let boot_services = system_table.boot_services();
    let runtime_services = system_table.runtime_services();
    //Print command
    writeln!(printer, ">{}", command_str);
    //Split command into iterator
    let mut args: Split<&str> = command_str.split(" ");
    let command: &str = match args.next(){
        Some(s) => s,
        None => {writeln!(printer, "Processor: No command entered."); return 0;}
    };
    //Assess command (NEW)
    match command {
        command if command.eq_ignore_ascii_case("boot")     => {writeln!       (printer, "Processor: Boot sequence requested."); return 0x01;},
        command if command.eq_ignore_ascii_case("shutdown") => {writeln!       (printer, "Processor: Shutdown requested.");      return 0x02;},
        command if command.eq_ignore_ascii_case("panic")    => {writeln!       (printer, "Processor: Panic requested.");         return 0x03;},
        command if command.eq_ignore_ascii_case("time")     => {command_time   (printer, runtime_services, &mut args);           return 0x00;},
        command if command.eq_ignore_ascii_case("memmap")   => {command_memmap (printer, boot_services, &mut args);              return 0x00;},
        command if command.eq_ignore_ascii_case("memread")  => {command_memread(printer, args);                                  return 0x00;},
        command if command.eq_ignore_ascii_case("crread")   => {command_crread (printer, &mut args);                             return 0x00;},
        command                                             => {writeln!       (printer, "Processor: Unrecognized command.");    return 0x00;},
    }
}

//Display the time
fn command_time(printer: &mut dyn Write, runtime_services: &RuntimeServices, args: &mut Split<&str>) {
    //Help String
    let help: &str = "\"TIME: Display the computer's Unix time in UTC.\":\n\
                      time\n\
                      Command takes no arguments.\n";
    //Processing
    loop {
        let arg = match args.next() {
            Some(s) => s,
            None => {break;},
        };
        if arg.starts_with("-") {writeln!(printer, "Invalid flag: {}.\n{}",     arg, help); return;}
        else                    {writeln!(printer, "Invalid argument: {}.\n{}", arg, help); return;}
    }
    //Time Display
    let t = runtime_services.get_time();
    match t {
        Ok(t) => {let t = t.log(); writeln!(printer, "{}-{:02}-{:02} {:02}:{:02}:{:02} UTC", t.year(), t.month(), t.day(), t.hour(), t.minute(), t.second());},
        Err(error) => {writeln!(printer, "Failed to retrieve time:\n{}", uefi_error_readout(error.status()));}
    }
}

//Display memory map contents to console
fn command_memmap(printer: &mut dyn Write, boot_services: &BootServices, args: &mut Split<&str>) {
    //Help String
    let help: &str = "\"MEMMAP: Display the contents of the UEFI memory map.\":\n\
                      memmap\n\
                      Command takes no arguments.\n";
    //Processing
    loop {
        let arg = match args.next() {
            Some(s) => s,
            None => {break;},
        };
        if arg.starts_with("-") {writeln!(printer, "Invalid flag: {}.\n{}",     arg, help); return;}
        else                    {writeln!(printer, "Invalid argument: {}.\n{}", arg, help); return;}
    }
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
            MemoryType::LOADER_CODE => {memory_type_text =           "LOADER CODE          "}
            MemoryType::LOADER_DATA => {memory_type_text =           "LOADER DATA          "}
            MemoryType::BOOT_SERVICES_CODE => {memory_type_text =    "BOOT SERVICES CODE   "}
            MemoryType::BOOT_SERVICES_DATA => {memory_type_text =    "BOOT SERVICES DATA   "}
            MemoryType::RUNTIME_SERVICES_CODE => {memory_type_text = "RUNTIME SERVICES CODE"}
            MemoryType::RUNTIME_SERVICES_DATA => {memory_type_text = "RUNTIME SERVICES DATA"}
            MemoryType::CONVENTIONAL => {memory_type_text =          "CONVENTIONAL         "}
            MemoryType::UNUSABLE => {memory_type_text =              "UNUSABLE             "}
            MemoryType::ACPI_RECLAIM => {memory_type_text =          "ACPI RECLAIM         "}
            MemoryType::ACPI_NON_VOLATILE => {memory_type_text =     "ACPI NON VOLATILE    "}
            MemoryType::MMIO => {memory_type_text =                  "MEMORY MAPPED IO     "}
            MemoryType::MMIO_PORT_SPACE => {memory_type_text =       "MEMORY MAPPED PORT   "}
            MemoryType::PAL_CODE => {memory_type_text =              "PROCESSOR MEMORY     "}
            MemoryType::PERSISTENT_MEMORY => {memory_type_text =     "PERSISTENT           "}
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
    enum Width {W8, W16, W24, W32, W48, W64}
    enum Radix {B, O, D, X}
    let mut a: (bool, usize) = (false, 0);
    let mut w: (bool, Width) = (false, Width::W8);
    let mut r: (bool, Radix) = (false, Radix::X);
    let mut c: (bool, usize) = (false, 1);
    //Help String
    let help: &str = "MEMREAD     : Display an aribtrary portion of memory.\n\
                      crread      -a [ address ]  ( -w [ width ] ) ( -r [ radix ] ) ( -c [ count ] ) \n\
                      -a          : Read from 'address'.\n\
                       address    : Hexidecimal number.\n\
                      -w          : Read using and display with the given bit 'width'.\n\
                       width      : '8', '16', '24', '32', '48', or '64'. Default of '8'.\n\
                      -r          : Display with the given 'radix'.\n\
                       radix      : 'b'inary, 'o'ctal, 'd'ecimal, or he'x'idecimal. Default of 'x'.\n\
                      -c          : Sequentially repeat the read with the given 'count'.\n\
                       count      : Hexidecimal number. Default of 1.";
    //Processing
    loop {
        let arg = match args.next() {
            Some(s) => s,
            None => {break;},
        };
        if arg.starts_with("-") { match arg {
            arg if arg.eq_ignore_ascii_case("-a")  => {if cr0||cr2||cr3||cr4||efer {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr0 = true; cr2 = true; cr3 = true; cr4 = true; efer = true;}},
            arg if arg.eq_ignore_ascii_case("-w")  => {if cr0                      {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr0 = true}},
            arg if arg.eq_ignore_ascii_case("-r")  => {if cr2                      {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr2 = true}},
            arg if arg.eq_ignore_ascii_case("-c")  => {if cr3                      {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr3 = true}},
            arg if arg.eq_ignore_ascii_case("-cr4")  => {if cr4                      {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr4 = true}},
            arg if arg.eq_ignore_ascii_case("-efer") => {if efer                     {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {efer = true}},
            _ => {writeln!(printer, "Invalid flag: {}.\n{}",     arg, help); return;}
        }}
        else     {writeln!(printer, "Invalid argument: {}.\n{}", arg, help); return;}
    }
    //Check validity
    if !(cr0||cr2||cr3||cr4||efer) {writeln!(printer, "{}", help); return;}
    //Read subcommand from arguments
    let split:Vec<&str> = args.split(" ").collect();
    //Handle if number of arguments is incorrect
    if split.len() != 4 {
        writeln!(printer, "Incorrect number of arguments: {} (requires 3).", split.len());
        return;
    }
    //Read numbers from argument
    let size = match split[3].to_string().parse::<usize>(){Ok(i) => {i}, Err(_) => {writeln!(printer, "3rd argument not valid: {}", split[3]); return;}};
    //Read memory
    if split[1].eq_ignore_ascii_case("u64b"){
        let address = match usize::from_str_radix(split[2], 16){Ok(i) => {i}, Err(_) => {writeln!(printer, "2nd argument not valid: {}", split[2]); return;}} as *mut u8;
        unsafe{
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                writeln!(printer, "0x{:016X}: 0b{:064b}", address as usize + i*8, num);
            }
        }
        writeln!(printer, "{:p} {}", address, size);
    }
    else if split[1].eq_ignore_ascii_case("u64x"){
        let address = match usize::from_str_radix(split[2], 16){Ok(i) => {i}, Err(_) => {writeln!(printer, "2nd argument not valid: {}", split[2]); return;}} as *mut u8;
        unsafe {
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                writeln!(printer, "0x{:016X}: 0x{:016X}", address as usize + i*8, num);
            }
        }
        writeln!(printer, "{:p} {}", address, size);
    }
    else if split[1].eq_ignore_ascii_case("u64o"){
        let address = match usize::from_str_radix(split[2], 8){Ok(i) => {i}, Err(_) => {writeln!(printer, "2nd argument not valid: {}", split[2]); return;}} as *mut u8;
        unsafe {
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                writeln!(printer, "0o{:016o}: 0o{:022o}", address as usize + i*8, num);
            }
        }
        writeln!(printer, "{:p} {}", address, size);
    }
    else{
        writeln!(printer, "1st argument not valid: {}", split[1]);
    }
}

//Display contents of control registers
fn command_crread(printer: &mut dyn Write, args: &mut Split<&str>) {
    //Pre processing variables
    let mut cr0:  bool = false;
    let mut cr2:  bool = false;
    let mut cr3:  bool = false;
    let mut cr4:  bool = false;
    let mut efer: bool = false;
    //Help String
    let help: &str = "CRREAD      : Display the x86-64 control registers.\n\
                      crread      [ -all | -cr0 -cr2 -cr3 -cr4 -efer ]\n\
                      -all        : Display All Registers\n\
                      -cr0        : Display Control Register 0\n\
                      -cr2        : Display Control Register 2\n\
                      -cr3        : Display Control Register 3\n\
                      -cr4        : Display Control Register 4\n\
                      -efer       : Display Extended Feature Enable Register";
    //Processing
    loop {
        let arg = match args.next() {
            Some(s) => s,
            None => {break;},
        };
        if arg.starts_with("-") { match arg {
            arg if arg.eq_ignore_ascii_case("-all")  => {if cr0||cr2||cr3||cr4||efer {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr0 = true; cr2 = true; cr3 = true; cr4 = true; efer = true;}},
            arg if arg.eq_ignore_ascii_case("-cr0")  => {if cr0                      {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr0 = true}},
            arg if arg.eq_ignore_ascii_case("-cr2")  => {if cr2                      {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr2 = true}},
            arg if arg.eq_ignore_ascii_case("-cr3")  => {if cr3                      {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr3 = true}},
            arg if arg.eq_ignore_ascii_case("-cr4")  => {if cr4                      {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {cr4 = true}},
            arg if arg.eq_ignore_ascii_case("-efer") => {if efer                     {writeln!(printer, "Flag usage not valid.\n{}", help); return;} else {efer = true}},
            _ => {writeln!(printer, "Invalid flag: {}.\n{}",     arg, help); return;}
        }}
        else     {writeln!(printer, "Invalid argument: {}.\n{}", arg, help); return;}
    }
    //Check validity
    if !(cr0||cr2||cr3||cr4||efer) {writeln!(printer, "{}", help); return;}
    //Control Register Display
    if cr0  {writeln!(printer, "Control Register 0:\n  Flags:   0x{:016X}", Cr0::read().bits());}
    if cr2  {writeln!(printer, "Control Register 2:\n  Address: 0x{:016X}", Cr2::read().as_u64());}
    if cr3  {writeln!(printer, "Control Register 3:\n  Flags:   0b{:016X}\n  Address: 0x{:016X}", Cr3::read().1.bits(), Cr3::read().0.start_address());}
    if cr4  {writeln!(printer, "Control Register 4:\n  Flags:   0x{:016X}", Cr4::read().bits());}
    if efer {writeln!(printer, "Extended Feature Enable Register:\n  Flags:   0x{:016X}", Efer::read().bits());}
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
        return pointer_return
    }
    else {
        boot_services.allocate_pool(memory_type, size).warning_as_error().unwrap_or(ptr::null_mut())
    }
}

//Allocate memory which has been zeroed
unsafe fn allocate_page_zeroed(boot_services: &BootServices, memory_type: MemoryType) -> *mut u8 {
    let pointer = allocate_memory(boot_services, memory_type, PAGE_SIZE_4KIB, PAGE_SIZE_4KIB);
    for i in 0..PAGE_SIZE_4KIB{
        write_volatile(pointer.add(i), 0x00);
    }
    return pointer;
}

//Write page table entry
unsafe fn write_pte(pt_address: *mut u8, pte_address: *mut u8, offset: usize, align: usize, global: bool, large_size: bool, cache_disable: bool, write_through: bool, supervisor: bool, read_write: bool) -> bool {
    //Return if invalid addresses
    if pt_address  as usize % PAGE_SIZE_4KIB != 0              {return false;}
    if pte_address as usize % align          != 0              {return false;}
    if offset                                >= PAGE_NUMBER_1 {return false;}
    //Set flags
    let mut pte = pte_address as usize;
    if global        {pte |= 0b0001_0000_0000;}
    if large_size    {pte |= 0b0000_1000_0000;}
    if cache_disable {pte |= 0b0000_0001_0000;}
    if write_through {pte |= 0b0000_0000_1000;}
    if supervisor    {pte |= 0b0000_0000_0100;}
    if read_write    {pte |= 0b0000_0000_0010;}
                      pte |= 0b0000_0000_0001;
    //Convert to bytes
    let bytes = usize::to_le_bytes(pte | 0x001);
    //Write entry
    for i in 0..bytes.len(){
        write_volatile(pt_address.add(offset*8 + i), bytes[i]);
    }
    return true;
}

//Read page table entry
unsafe fn read_pte(pt_address: *mut u8, offset: usize) -> *mut u8 {
    let mut bytes: [u8;8] = [0;8];
    for i in 0..8{
        bytes[i] = read_volatile(pt_address.add(offset+i));
    }
    let num = usize::from_le_bytes(bytes);
    return ((num/PAGE_SIZE_4KIB) * PAGE_SIZE_4KIB) as *mut u8;
}

//Create a Level 3 Page Map From a Physical Offset Using 4KiB Pages
unsafe fn create_pml3_offset_4kib(boot_services:&BootServices, start_address: *mut u8, size: usize) -> *mut u8 {
    //Check Parameters
    if start_address as usize % PAGE_SIZE_4KIB !=0 {return 0 as *mut u8;}
    if size > PAGE_SIZE_512G {return 0 as *mut u8;}
    //New PML3
    let pml3 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
    //Lower Page Maps
    let mut pml2: *mut u8 = 0 as *mut u8;
    let mut pml1: *mut u8 = 0 as *mut u8;
    //Number of 4KiB pages
    let pages = size/PAGE_SIZE_4KIB + if size%PAGE_SIZE_4KIB != 0 {1} else {0};
    //Allocation Loop
    for i in 0..pages {
        if i%PAGE_NUMBER_1 == 0 {
            if i%PAGE_NUMBER_2 == 0 {
                //New PML2
                pml2 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
                write_pte(pml3, pml2, i/PAGE_NUMBER_2, PAGE_SIZE_4KIB, false, false, false, false, false, true);
            }
            //New PML1
            pml1 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
            write_pte(pml2, pml1, i/PAGE_NUMBER_1, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        }
        //New 4KiB Page
        write_pte(pml1, start_address.add(i*PAGE_SIZE_4KIB), i%PAGE_NUMBER_1, PAGE_SIZE_4KIB, false, false, false, false, false, true);
    }
    //Finish
    return pml3;
}

//Create a Level 3 Page Map From a Physical Offset Using 2MiB Pages
unsafe fn create_pml3_offset_2mib(boot_services: &BootServices, start_address: *mut u8, size: usize) -> *mut u8 {
    //Check Alignment
    if start_address as usize % PAGE_SIZE_2MIB !=0 {return 0 as *mut u8;}
    if size > PAGE_SIZE_512G {return 0 as *mut u8;}
    //New PML3
    let pml3 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
    //Lower Page Maps
    let mut pml2: *mut u8 = 0 as *mut u8;
    //Number of 2MiB Pages
    let pages = size/PAGE_SIZE_2MIB + if size%PAGE_SIZE_2MIB != 0 {1} else {0};
    //Allocation Loop
    for i in 0..pages {
        if i%PAGE_NUMBER_1 == 0 {
            //New PML2
            pml2 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
            write_pte(pml3, pml2, i/PAGE_NUMBER_1, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        }
        //New 2MiB Page
        write_pte(pml2, start_address.add(i*PAGE_SIZE_2MIB), i%PAGE_NUMBER_1, PAGE_SIZE_2MIB, false, true, false, false, false, true);
    }
    //Finish
    return pml3;
}

//Create a Level 3 Page Map from a Physical Offset Using 1GiB Pages
unsafe fn create_pml3_offset_1gib(boot_services: &BootServices, start_address: *mut u8, size: usize) -> *mut u8 {
    //Check Parameters
    if start_address as usize % PAGE_SIZE_4KIB !=0 {return 0 as *mut u8;}
    if size > PAGE_SIZE_512G {return 0 as *mut u8;}
    //New PML3
    let pml3 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
    //Number of 1GiB Pages
    let pages = size/PAGE_SIZE_1GIB + if size%PAGE_SIZE_1GIB != 0 {1} else {0};
    //Allocation Loop
    for i in 0..pages {
        //New 1GiB Page
        write_pte(pml3, start_address.add(i*PAGE_SIZE_1GIB), i, PAGE_SIZE_1GIB, false, true, false, false, false, true);
    }
    //Finish
    return pml3;
}
