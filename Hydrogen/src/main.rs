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
#![feature(abi_efiapi)]
#![feature(box_syntax)]
#![feature(asm)]
#![feature(bench_black_box)]
#![allow(unused_must_use)]

//External crates
extern crate rlibc;
extern crate alloc;

//Imports
use gluon::elf_file_header::ApplicationBinaryInterface;
use gluon::elf_file_header::InstructionSetArchitecture;
use gluon::elf_file_header::ObjectType;
use gluon::elf_program_header::ELFProgramHeader;
use gluon::elf_program_header::ProgramType;
use photon::*;
use gluon::*;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Write;
use core::intrinsics::copy_nonoverlapping;
use core::panic::PanicInfo;
use core::ptr;
use core::ptr::null;
use core::ptr::null_mut;
use core::ptr::read_volatile;
use core::ptr::write_volatile;
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
const HYDROGEN_VERSION: & str = "vDEV-2021-08-13"; //CURRENT VERSION OF BOOTLOADER
pub const PIXL_SCRN_X_DIM:       usize                 = 1920;                     //PIXEL WIDTH OF SCREEN
pub const PIXL_SCRN_Y_DIM:       usize                 = 1080;                     //PIXEL HEIGHT OF SCREEN
pub const PIXL_SCRN_B_DEP:       usize                 = 4;                        //PIXEL BIT DEPTH
pub const COLR_PRBLK:            [u8; PIXL_SCRN_B_DEP] = [0x00, 0x00, 0x00, 0x00]; //COLOR PURE BLACK
pub const COLR_PRRED:            [u8; PIXL_SCRN_B_DEP] = [0x00, 0x00, 0xFF, 0x00]; //COLOR PURE RED
pub const COLR_PRGRN:            [u8; PIXL_SCRN_B_DEP] = [0x00, 0xFF, 0x00, 0x00]; //COLOR PURE GREEN
pub const COLR_PRBLU:            [u8; PIXL_SCRN_B_DEP] = [0xFF, 0x00, 0x00, 0x00]; //COLOR PURE BLUE
pub const COLR_PRWHT:            [u8; PIXL_SCRN_B_DEP] = [0xFF, 0xFF, 0xFF, 0x00]; //COLOR PURE WHITE
pub const CHAR_SCRN_X_DIM:       usize                 = 120;                      //TEXT MODE WIDTH OF ENTIRE SCREEN
pub const CHAR_SCRN_Y_DIM:       usize                 = 67;                       //TEXT MODE HEIGHT OF ENTIRE SCREEN
pub const CHAR_PRNT_X_POS:       usize                 = 1;                        //TEXT MODE HORIZONTAL POSITION OF PRINT RESULT WINDOW
pub const CHAR_PRNT_Y_POS:       usize                 = 2;                        //TEXT MODE VERTICAL POSITION OF PRINT RESULT WINDOW
pub const CHAR_PRNT_X_DIM:       usize                 = 118;                      //TEXT MODE WIDTH OF PRINT RESULT WINDOW
pub const CHAR_PRNT_Y_DIM_DSP:   usize                 = 62;                       //TEXT MODE HEIGHT OF PRINT RESULT WINDOW ON SCREEN
pub const CHAR_PRNT_Y_DIM_MEM:   usize                 = 400;                      //TEXT MODE HEIGHT OF PRINT RESULT WINDOW IN MEMORY
pub const CHAR_INPT_X_POS:       usize                 = 1;                        //TEXT MODE HORIZONTAL POSITION OF INPUT WINDOW
pub const CHAR_INPT_Y_POS:       usize                 = 65;                       //TEXT MODE VERTICAL POSITION OF INPUT WINDOW
pub const CHAR_INPT_X_DIM:       usize                 = 118;                      //TEXT MODE WIDTH OF INPUT WINDOW
pub const CHAR_INPT_Y_DIM_DSP:   usize                 = 1;                        //TEXT MODE HEIGHT OF INPUT WINDOW
pub const CHAR_INPT_Y_DIM_MEM:   usize                 = 1;                        //TEXT MODE HEIGHT OF INPUT WINDOW IN MEMORY

type FullScreen = Screen<
    PIXL_SCRN_Y_DIM,     PIXL_SCRN_X_DIM,     PIXL_SCRN_B_DEP, CHAR_SCRN_Y_DIM, CHAR_SCRN_X_DIM,
    CHAR_PRNT_Y_DIM_MEM, CHAR_PRNT_Y_DIM_DSP, CHAR_PRNT_X_DIM, CHAR_PRNT_Y_POS, CHAR_PRNT_X_POS,
    CHAR_PRNT_Y_DIM_MEM, CHAR_INPT_Y_DIM_DSP, CHAR_INPT_X_DIM, CHAR_INPT_Y_POS, CHAR_INPT_X_POS,
>;

// MAIN
//Main Entry Point After UEFI Boot
#[entry]
fn efi_main(_handle: Handle, system_table_boot: SystemTable<Boot>) -> Status {
    // UEFI INITILIZATION
    //Utilities initialization (Alloc)
    uefi_services::init(&system_table_boot).expect_success("UEFI Initialization: Utilities initialization failed.");
    let boot_services = system_table_boot.boot_services();
    //Console reset
    system_table_boot.stdout().reset(false).expect_success("Console reset failed");
    //Watchdog Timer shutoff
    boot_services.set_watchdog_timer(0, 0x10000, Some(&mut {&mut [0x0058u16, 0x0000u16]}[..])).expect_success("UEFI Initialization: Watchdog Timer shutoff failed.");
    //Graphics Output Protocol initialization
    let graphics_output_protocol = match boot_services.locate_protocol::<GraphicsOutput>() {
        Ok(gop) => gop,
        Err(error) => panic!("UEFI Initialization: Graphics Output Protocol not found.")
    };
    let graphics_output_protocol = graphics_output_protocol.expect("Graphics Output Protocol initialization failed at unsafe cell");
    let graphics_output_protocol = unsafe {&mut *graphics_output_protocol.get()};
    let graphics_frame_pointer = graphics_output_protocol.frame_buffer().as_mut_ptr();
    //Graphics Output Protocol set graphics mode
    set_graphics_mode(graphics_output_protocol);
    let _st = graphics_output_protocol.current_mode_info().stride();
    let _pf = graphics_output_protocol.current_mode_info().pixel_format();
    let _s = graphics_output_protocol.frame_buffer().size();
    //Simple File System initialization
    let simple_file_system = match boot_services.locate_protocol::<SimpleFileSystem>() {
        Ok(sfs) => sfs,
        Err(_) => panic!("Simple File System initialization failed at completion")
    };
    let simple_file_system = simple_file_system.expect("Simjple File System initialization failed at unsafe cell");
    let simple_file_system = unsafe {&mut *simple_file_system.get()};
    //Input initialization
    let input = match boot_services.locate_protocol::<Input>() {
        Ok(ink) => ink,
        Err(_) => panic!("Input initialization failed at completion")
    };
    let input = input.expect("Input initialization failed at unsafe cell");
    let input = unsafe {&mut *input.get()};

    // FILE READ SYSTEM
    //Wrapper Struct
    struct FileWrapper<'a>{
        file: &'a mut RegularFile,
    }
    //Locational Read Implementation
    impl<'a> LocationalRead for FileWrapper<'a>{
        fn read(&mut self, offset: u64, buffer: &mut [u8]) -> Result<(), &'static str> {
            self.file.set_position(offset);
            match self.file.read(buffer) {
                Ok(completion) => {
                    let size = completion.unwrap(); 
                    if size == buffer.len(){
                        Ok(())
                    } 
                    else {
                        Err("UEFI File Read: Buffer exceeds end of file.")
                    }
                },
                Err(error) => Err(uefi_error_readout(error.status())),
            }
        }
    }

    // GRAPHICS SETUP
    //Screen variables
    /*let     screen_physical:  *mut u8                                               = graphics_frame_pointer;
    let mut screen_charframe:      [Character; CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM]     = [Character::new(' ', COLR_PRWHT, COLR_PRBLK); CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM];
    //Input Window variables
    let mut input_stack:           [Character; CHAR_INPT_X_DIM*CHAR_INPT_Y_DIM_MEM] = [Character::new(' ', COLR_PRWHT, COLR_PRBLK); CHAR_INPT_X_DIM*CHAR_INPT_Y_DIM_MEM];
    let mut input_p:               usize                                            = 0;
    //Print Result Window variables
    let mut print_buffer:          [Character; CHAR_PRNT_X_DIM*CHAR_PRNT_Y_DIM_MEM] = [Character::new(' ', COLR_PRWHT, COLR_PRBLK); CHAR_PRNT_X_DIM*CHAR_PRNT_Y_DIM_MEM];
    let mut print_y:               usize                                            = CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM;
    let mut print_x:               usize                                            = 0;
    //Screen struct
    let mut screen:Screen = Screen{
        screen:       screen_physical,
        charframe: &mut screen_charframe,
        input_buffer:      &mut input_stack,
        input_p:          &mut input_p,
        print_buffer:     &mut print_buffer,
        print_y:          &mut print_y,
        print_x:          &mut print_x,
        print_fore:       &mut COLR_PRWHT,
        print_back:       &mut COLR_PRBLK,
    };*/
    let blue =  [0xFF, 0x00, 0x00, 0x00];
    let white = [0xFF, 0xFF, 0xFF, 0x00];
    let black = [0x00, 0x00, 0x00, 0x00];
    let whitespace = Character::<4>::new(' ', white, black);
    let bluespace = Character::<4>::new(' ', blue, black);
    let screen = FullScreen::new(graphics_frame_pointer, Character::<4>::new(' ', white, black));
    //User Interface initialization
    screen.draw_hline(CHAR_PRNT_Y_POS-1, 0,                 CHAR_SCRN_X_DIM-1,  bluespace);
    screen.draw_hline(CHAR_INPT_Y_POS-1, 0,                 CHAR_SCRN_X_DIM-1,  bluespace);
    screen.draw_hline(CHAR_INPT_Y_POS+1, 0,                 CHAR_SCRN_X_DIM-1,  bluespace);
    screen.draw_vline(0,                 CHAR_PRNT_Y_POS-1, CHAR_INPT_Y_POS+1,  bluespace);
    screen.draw_vline(CHAR_SCRN_X_DIM-1, CHAR_PRNT_Y_POS-1, CHAR_INPT_Y_POS+1,  bluespace);
    screen.draw_string("NOBLE OS",            0, 0,                                             bluespace);
    screen.draw_string("HYDROGEN BOOTLOADER", 0, CHAR_SCRN_X_DIM - 20 - HYDROGEN_VERSION.len(), bluespace);
    screen.draw_string(HYDROGEN_VERSION,      0, CHAR_SCRN_X_DIM -      HYDROGEN_VERSION.len(), bluespace);
    screen.characterframe_render();
    writeln!(screen, "Hydrogen Bootloader     {}", HYDROGEN_VERSION);
    writeln!(screen, "Photon Graphics Library {}", PHOTON_VERSION);
    writeln!(screen, "Gluon Memory Library    {}", GLUON_VERSION);
    writeln!(screen, "Frame Buffer Location:  0o{0:016o} 0x{0:016X}", graphics_frame_pointer as usize);

    // LOAD KERNEL
    //Find kernel on disk
    let mut sfs_dir_root = simple_file_system.open_volume().expect_success("File system root failed to open.");
    let sfs_kernel_handle = sfs_dir_root.open("noble", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"noble\".").
        open("helium", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"helium\".").
        open("x86-64.elf", FileMode::Read, FileAttribute::empty()).expect_success("File system kernel open failed at \"x86-64.elf\".");
    let mut sfs_kernel = unsafe { RegularFile::new(sfs_kernel_handle) };
    writeln!(screen, "Found kernel on file system.");
    //Read kernel file
    let mut sfs_kernel_wrap = FileWrapper{file: &mut sfs_kernel};
    let mut kernel_program_buffer = [ELFProgramHeader::default(); 10];
    let kernel = match ELFFile::new(&mut sfs_kernel_wrap, &mut kernel_program_buffer) {
        Ok(elffile) =>  {writeln!(screen, "New ELF Reader System Success."); elffile},
        Err(error) => {writeln!(screen, "New ELF Reader System Failure: {}", error); panic!()},
    };
    //Check ELF header validity
    if kernel.file_header.binary_interface         != ApplicationBinaryInterface::None     {writeln!(screen, "Kernel load: Incorrect Application Binary Interface (ei_osabi). Should be SystemV/None (0x00)."); panic!();}
    if kernel.file_header.binary_interface_version != 0x00                                 {writeln!(screen, "Kernel load: Incorrect Application Binary Interface Version (ei_abiversion). Should be None (0x00)."); panic!();}
    if kernel.file_header.architecture             != InstructionSetArchitecture::EmX86_64 {writeln!(screen, "Kernel load: Incorrect Instruction Set Architecture (e_machine). Should be x86-64 (0x3E)."); panic!();}
    if kernel.file_header.object_type              != ObjectType::Shared                   {writeln!(screen, "Kernel Load: Incorrect Object Type (e_type). Should be Dynamic (0x03)."); panic!()}
    //Print ELF header info
    {
        writeln!(screen, "Kernel Entry Point:                 0x{:04X}", kernel.file_header.entry_point);
        writeln!(screen, "Kernel Program Header Offset:       0x{:04X}", kernel.file_header.program_header_offset);
        writeln!(screen, "Kernel Section Header Offset:       0x{:04X}", kernel.file_header.section_header_offset);
        writeln!(screen, "Kernel ELF Header Size:             0x{:04X}", kernel.file_header.header_size);
        writeln!(screen, "Kernel Program Header Entry Size:   0x{:04X}", kernel.file_header.program_header_entry_size);
        writeln!(screen, "Kernel Program Header Number:       0x{:04X}", kernel.file_header.program_header_number);
        writeln!(screen, "Kernel Section Header Entry Size:   0x{:04X}", kernel.file_header.section_header_entry_size);
        writeln!(screen, "Kernel Section Header Number:       0x{:04X}", kernel.file_header.section_header_number);
        writeln!(screen, "Kernel Section Header String Index: 0x{:04X}", kernel.file_header.string_section_index);
        writeln!(screen, "Kernel Code Size:                   0x{:04X}", kernel.program_headers.len());
    }
    //Allocate memory for code
    let kernel_stack_size = PAGE_SIZE_2MIB;
    writeln!(screen, "Kernel Memory Size (new): {}", kernel.memory_size());
    let kernel_total_size = kernel.memory_size() as usize + kernel_stack_size;
    let kernl_ptr_phys = unsafe { allocate_memory(boot_services, MemoryType::LOADER_CODE, kernel_total_size, PAGE_SIZE_4KIB as usize) };
    //Load code into memory
    for program_header in kernel.program_headers {
        if program_header.program_type == ProgramType::Loadable {
            let mut buf:Vec<u8> = vec![0; program_header.file_size as usize];
            kernel.file.read(program_header.file_offset, &mut buf);
            unsafe {copy_nonoverlapping(buf.as_ptr(), kernl_ptr_phys.add(program_header.virtual_address as usize), program_header.memory_size as usize);}
        }
    }
    //Dynamic Relocation

    // BOOT LOAD
    let kernl_efn: extern "sysv64" fn() -> !;
    let pml4_knenv: *mut u8;
    unsafe{
        // PAGE TABLES
        //Page Map Level 4: Kernel Environment
        pml4_knenv = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
        writeln!(screen, "PML4 KNENV: 0o{0:016o} 0x{0:016X}", pml4_knenv as usize);
        //Page Map Level 4: EFI Boot
        let pml4_efibt:*mut u8 = Cr3::read().0.start_address().as_u64() as *mut u8;
        writeln!(screen, "PML4 EFIBT: 0o{0:016o} 0x{0:016X}", pml4_efibt as usize);
        //Page Map Level 3: EFI Boot Physical Memory
        let pml3_efiph:*mut u8 = read_pte(pml4_efibt, 0);
        writeln!(screen, "PML3 EFIPH: 0o{0:016o} 0x{0:016X}", pml3_efiph as usize);
        //Page Map Level 3: Operating System Initialized Physical Memory
        let pml3_osiph:*mut u8 = create_pml3_offset_1gib(boot_services, 0 as *mut u8, PAGE_SIZE_512G);
        writeln!(screen, "PML3 OSIPH: 0o{0:016o} 0x{0:016X}", pml3_osiph as usize);
        //Page Map Level 3: Kernel
        let pml3_kernl = create_pml3_offset_4kib(boot_services, kernl_ptr_phys, kernel_total_size);
        writeln!(screen, "PML3 KERNL: 0o{0:016o} 0x{0:016X}", pml3_kernl as usize);
        //Page Map Level 3: Frame Buffer
        let pml3_frame = create_pml3_offset_2mib(boot_services, graphics_frame_pointer, PIXL_SCRN_Y_DIM*PIXL_SCRN_X_DIM*PIXL_SCRN_B_DEP);
        writeln!(screen, "PML3 FRAME: 0o{0:016o} 0x{0:016X}", pml3_frame as usize);
        //Write PML4 Entries
        write_pte(pml4_knenv, pml3_efiph, PHYSM_PHYS_OCT, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        write_pte(pml4_knenv, pml3_kernl, KERNL_VIRT_OCT, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        write_pte(pml4_knenv, pml3_frame, FRAME_VIRT_OCT, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        write_pte(pml4_knenv, pml3_osiph, PHYSM_VIRT_OCT, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        write_pte(pml4_knenv, pml4_knenv, PGMAP_VIRT_OCT, PAGE_SIZE_4KIB, false, false, false, false, false, true);
    }

    // COMMAND LINE
    //Enter Read-Evaluate-Print Loop
    loop{
        //Wait for key to be pressed
        boot_services.wait_for_event(&mut [input.wait_for_key_event()]).expect_success("Boot services event wait failed");
        //Check input key
        let input_key = input.read_key().expect_success("Key input failed").unwrap();
        match input_key{
            //Printable Key
            Key::Printable(input_char16) => {
                //Convert to usable type
                let input_char = char::from(input_char16);
                //User has hit enter
                if input_char == '\r'{
                    //Return to bottom of screen
                    screen.print_y = CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP;
                    //Execute command and reset input stack
                    let command = &screen.input_as_chararray(' ')[0..screen.input_p].iter().collect::<String>();
                    let boot_command_return = command_processor(&mut screen, &system_table_boot, command);
                    screen.input_flush(Character::new(' ', COLR_PRWHT, COLR_PRBLK));
                    //Check return code
                    if boot_command_return != 0 {
                        //Boot sequence
                        if boot_command_return == 1 {
                            break;
                        }
                        //Shutdown sequence
                        else if boot_command_return == 2 {
                            system_table_boot.boot_services().stall(5_000_000);
                            system_table_boot.runtime_services().reset(ResetType::Shutdown, Status(0), None);
                        }
                    }
                }
                //User has typed a character
                else {
                    screen.character_input_draw_render(Character::new(input_char, white, black), whitespace);
                }
            }
            //Modifier or Control Key
            Key::Special(scancode) =>{
                if scancode == ScanCode::UP && screen.print_y > 0 {
                    screen.print_y -= 1;
                    screen.printbuffer_draw_render();
                }
                else if scancode == ScanCode::DOWN && screen.print_y < CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP{
                    screen.print_y += 1;
                    screen.printbuffer_draw_render();
                }
                else if scancode == ScanCode::END{
                    screen.print_y = CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP;
                    screen.printbuffer_draw_render();
                }
                else if scancode == ScanCode::PAGE_UP{
                    screen.print_y = if screen.print_y > CHAR_PRNT_Y_DIM_DSP {screen.print_y - CHAR_PRNT_Y_DIM_DSP} else {0};
                    screen.printbuffer_draw_render();
                }
                else if scancode == ScanCode::PAGE_DOWN{
                    screen.print_y = if screen.print_y < CHAR_PRNT_Y_DIM_MEM - 2*CHAR_PRNT_Y_DIM_DSP {screen.print_y + CHAR_PRNT_Y_DIM_DSP} else {CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP};
                    screen.printbuffer_draw_render();
                }
            }
        }
    }
    writeln!(screen, "Simple REPL exited.");

    // EXIT BOOT SERVICES
    let mut memory_map_buffer = [0; 10000];
    let (_table_runtime, _esi) = system_table_boot.exit_boot_services(_handle, &mut memory_map_buffer).expect_success("Boot services exit failed");
    exit_boot_services();
    writeln!(screen, "Boot Services exited.");

    // ENTER KERNEL
    unsafe{asm!(
        "MOV R14, {stack}",
        "MOV R15, {entry}",
        "MOV CR3, {pagemap}",
        "MOV RSP, R14",
        "JMP R15",

        stack = in(reg) KERNL_VIRT_PTR as u64 + kernel_total_size as u64,
        entry = in(reg) KERNL_VIRT_PTR as u64 + kernel.file_header.entry_point,
        pagemap = in(reg) pml4_knenv as u64,
        options(nostack)
    );}

    // HALT COMPUTER
    writeln!(screen, "Halt reached.");
    unsafe {asm!("HLT");}
    loop {}
}


// PANIC HANDLER
//Panic State Variables
//static mut panic_screen_pointer: *mut Screen = 0 as *mut Screen;

//Panic Handler
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    unsafe {
        //if panic_screen_pointer != null_mut(){
        //    let panic_screen: Screen = panic_screen_pointer.try;
        //}
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
            info.resolution() == (PIXL_SCRN_X_DIM, PIXL_SCRN_Y_DIM)
        }).unwrap();
    gop.set_mode(&mode).expect_success("Graphics Output Protocol set mode failed.");
}


// COMMAND PROCESSOR
//Evaluate and execute a bootloader command and return a code
fn command_processor(screen: &mut FullScreen, system_table: &SystemTable<Boot>, command: &str) -> u8 {
    //Print command
    writeln!(screen, ">{}", command);
    //Assess command
    if command.eq_ignore_ascii_case("time"){
        //Time
        writeln!(screen, ">{}", match system_table.runtime_services().get_time(){
            Ok(v) => {let v = v.log(); format!{"{}-{:02}-{:02} {:02}:{:02}:{:02} UTC", v.year(), v.month(), v.day(), v.hour(), v.minute(), v.second()}},
            Err(e) => format!("Command failed: {:?}", e)});
    }
    else if command.eq_ignore_ascii_case("mem"){
        //Memory Map
        command_mem(screen, system_table.boot_services());
    }
    else if command.eq_ignore_ascii_case("shutdown"){
        //Shutdown
        writeln!(screen, "Shutdown sequence started.");
        return 0x02;
    }
    else if command.eq_ignore_ascii_case("boot"){
        //Boot
        writeln!(screen, "Boot sequence started.");
        return 0x01;
    }
    else if command.eq_ignore_ascii_case("crd"){
        //Control Register Display
        writeln!(screen, "CR0:  {:016x}", Cr0::read().bits());
        writeln!(screen, "CR2:  {:016x}", Cr2::read().as_u64());
        writeln!(screen, "CR3A: {:016x}", Cr3::read().0.start_address());
        writeln!(screen, "CR3F: {:016x}", Cr3::read().1.bits());
        writeln!(screen, "CR4:  {:016x}", Cr4::read().bits());
        writeln!(screen, "EFER: {:016x}", Efer::read().bits());
    }
    else if command.starts_with("peek "){
        //Peek Memory
        command_peek(screen, command);
    }
    else{
        //No Result
        writeln!(screen, "Command not entered properly.");
    }
    //Return to Command Line
    return 0x00;
}

//Display memory map contents to console
fn command_mem(screen: &mut FullScreen, boot_service: &BootServices){
    //Estimated map size
    let map_size = boot_service.memory_map_size();
    writeln!(screen, "Map size: {}", map_size);
    //Build a buffer big enough to handle the memory map
    let mut buffer = vec![0;map_size+512];
    writeln!(screen, "Buffer len: {}", buffer.len());
    writeln!(screen, "Buffer cap: {}", buffer.capacity());
    //Read memory map into buffer
    let (_k, description_iterator) = boot_service
        .memory_map(&mut buffer)
        .expect_success("UEFI Memory Map retrieval failed.");
    let descriptors = description_iterator.copied().collect::<Vec<_>>();
    //Handle if memory map appears empty
    if descriptors.is_empty() {
        writeln!(screen, "UEFI Memory Map is empty.");
        return;
    }
    //Print memory map
    writeln!(screen, "Usable ranges: {}", descriptors.len());
    for descriptor in descriptors{
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
        writeln!(screen, "{}: {:016x}-{:016x} ({:8}KiB / {:8}Pg)", memory_type_text, descriptor.phys_start, end_address, size/1024, size_pages);
    }
}

//Display the raw contents of a part of memory
fn command_peek(screen: &mut FullScreen, command: &str) {
    //Read subcommand from arguments
    let split:Vec<&str> = command.split(" ").collect();
    //Handle if number of arguments is incorrect
    if split.len() != 4 {
        writeln!(screen, "Incorrect number of arguments: {} (requires 3).", split.len());
        return;
    }
    //Read numbers from argument
    let size = match split[3].to_string().parse::<usize>(){Ok(i) => {i}, Err(_) => {writeln!(screen, "3rd argument not valid: {}", split[3]); return;}};
    //Read memory
    if split[1].eq_ignore_ascii_case("u64b"){
        let address = match usize::from_str_radix(split[2], 16){Ok(i) => {i}, Err(_) => {writeln!(screen, "2nd argument not valid: {}", split[2]); return;}} as *mut u8;
        unsafe{
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                writeln!(screen, "0x{:016X}: 0b{:064b}", address as usize + i*8, num);
            }
        }
        writeln!(screen, "{:p} {}", address, size);
    }
    else if split[1].eq_ignore_ascii_case("u64x"){
        let address = match usize::from_str_radix(split[2], 16){Ok(i) => {i}, Err(_) => {writeln!(screen, "2nd argument not valid: {}", split[2]); return;}} as *mut u8;
        unsafe {
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                writeln!(screen, "0x{:016X}: 0x{:016X}", address as usize + i*8, num);
            }
        }
        writeln!(screen, "{:p} {}", address, size);
    }
    else if split[1].eq_ignore_ascii_case("u64o"){
        let address = match usize::from_str_radix(split[2], 8){Ok(i) => {i}, Err(_) => {writeln!(screen, "2nd argument not valid: {}", split[2]); return;}} as *mut u8;
        unsafe {
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                writeln!(screen, "0o{:016o}: 0o{:022o}", address as usize + i*8, num);
            }
        }
        writeln!(screen, "{:p} {}", address, size);
    }
    else{
        writeln!(screen, "1st argument not valid: {}", split[1]);
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
    if offset                                >= PAGE_NMBR_LVL1 {return false;}
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
        if i%PAGE_NMBR_LVL1 == 0 {
            if i%PAGE_NMBR_LVL2 == 0 {
                //New PML2
                pml2 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
                write_pte(pml3, pml2, i/PAGE_NMBR_LVL2, PAGE_SIZE_4KIB, false, false, false, false, false, true);
            }
            //New PML1
            pml1 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
            write_pte(pml2, pml1, i/PAGE_NMBR_LVL1, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        }
        //New 4KiB Page
        write_pte(pml1, start_address.add(i*PAGE_SIZE_4KIB), i%PAGE_NMBR_LVL1, PAGE_SIZE_4KIB, false, false, false, false, false, true);
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
        if i%PAGE_NMBR_LVL1 == 0 {
            //New PML2
            pml2 = allocate_page_zeroed(boot_services, MemoryType::LOADER_DATA);
            write_pte(pml3, pml2, i/PAGE_NMBR_LVL1, PAGE_SIZE_4KIB, false, false, false, false, false, true);
        }
        //New 2MiB Page
        write_pte(pml2, start_address.add(i*PAGE_SIZE_2MIB), i%PAGE_NMBR_LVL1, PAGE_SIZE_2MIB, false, true, false, false, false, true);
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

