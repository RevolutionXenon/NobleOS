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
use alloc::{
    format, 
    string::{
        String, 
        ToString
    }, 
    vec, 
    vec::{
        Vec
    }
};
use core::{
    convert::{
        TryInto
    }, 
    fmt::{
        Write
    }, 
    intrinsics::{
        copy_nonoverlapping
    },
    mem::{
        transmute
    },
    ptr,
    ptr::{
        write_volatile
    }
};
use uefi::{
    alloc::{
        exit_boot_services
    },
    prelude::*,
    proto::{
        console::{
            gop::{
                GraphicsOutput, 
                Mode
            },
            text::{
                Input,
                Key,
                ScanCode
            }
        }, 
        media::{
            file::{
                File, 
                FileAttribute, 
                FileMode, 
                RegularFile
            }, 
            fs::{
                SimpleFileSystem
            }
        }
    },
    table::{
        boot::{
            MemoryType
        },
        runtime::{
            ResetType
        }
    }
};
use x86_64::{
    registers::{
        control::*
    }
};
use photon::*;

//Constants
const EFI_PAGE_SIZE: u64 = 0x1000;           //MEMORY PAGE SIZE (4KiB)
const CURRENT_VERSION: &str = "v2021-08-05"; //CURRENT VERSION OF BOOTLOADER


// MAIN
//Main entry point after firmware boot
#[entry]
fn efi_main(_handle: Handle, system_table_boot: SystemTable<Boot>) -> Status {
    // UEFI INITILIZATION
    //Utilities initialization (Alloc & Logger)
    uefi_services::init(&system_table_boot).expect_success("Utilities initialization failed");
    let boot_services = system_table_boot.boot_services();
    //Console reset
    system_table_boot.stdout().reset(false).expect_success("Console reset failed");
    //Watchdog Timer shutoff
    boot_services.set_watchdog_timer(0, 0x10000, Some(&mut {&mut [0x0058u16, 0x0000u16]}[..])).expect_success("Watchdog Timer shutoff failed");
    //Graphics Output Protocol initialization
    let graphics_output_protocol = match boot_services.locate_protocol::<GraphicsOutput>() {
        Ok(gop) => gop,
        Err(_) => panic!("Graphics Output Protocol initialization failed at completion")
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
    let simple_file_system = simple_file_system.expect("Simple File System initialization failed at unsafe cell");
    let simple_file_system = unsafe {&mut *simple_file_system.get()};
    //Input initialization
    let input = match boot_services.locate_protocol::<Input>() {
        Ok(ink) => ink,
        Err(_) => panic!("Input initialization failed at completion")
    };
    let input = input.expect("Input initialization failed at unsafe cell");
    let input = unsafe {&mut *input.get()};

    // GRAPHICS SETUP
    //Screen variables
    let     screen_physical:  *mut u8                                               = graphics_frame_pointer;
    let mut screen_charframe:      [Character; CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM]     = [Character::new(' ', COLR_WHITE, COLR_BLACK); CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM];
    //Input Window variables
    let mut input_stack:           [Character; CHAR_INPT_X_DIM*CHAR_INPT_Y_DIM_MEM] = [Character::new(' ', COLR_WHITE, COLR_BLACK); CHAR_INPT_X_DIM*CHAR_INPT_Y_DIM_MEM];
    let mut input_p:               usize                                            = 0;
    //Print Result Window variables
    let mut print_buffer:          [Character; CHAR_PRNT_X_DIM*CHAR_PRNT_Y_DIM_MEM] = [Character::new(' ', COLR_WHITE, COLR_BLACK); CHAR_PRNT_X_DIM*CHAR_PRNT_Y_DIM_MEM];
    let mut print_y:               usize                                            = CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP;
    let mut print_x:               usize                                            = 0;
    //Screen struct
    let mut screen:Screen = Screen{
        screen_physical:       screen_physical,
        screen_charframe: &mut screen_charframe,
        input_stack:      &mut input_stack,
        input_p:          &mut input_p,
        print_buffer:     &mut print_buffer,
        print_y:          &mut print_y,
        print_x:          &mut print_x,
    };
    //Wait 2 seconds
    system_table_boot.boot_services().stall(2_000_000);
    //User Interface initialization
    screen.draw_hline( CHAR_PRNT_Y_POS-1, 0,                 CHAR_SCRN_X_DIM-1,  COLR_PRRED, COLR_BLACK);
    screen.draw_hline( CHAR_INPT_Y_POS-1, 0,                 CHAR_SCRN_X_DIM-1,  COLR_PRRED, COLR_BLACK);
    screen.draw_hline( CHAR_INPT_Y_POS+1, 0,                 CHAR_SCRN_X_DIM-1,  COLR_PRRED, COLR_BLACK);
    screen.draw_vline( 0,                 CHAR_PRNT_Y_POS-1, CHAR_INPT_Y_POS+1,  COLR_PRRED, COLR_BLACK);
    screen.draw_vline( CHAR_SCRN_X_DIM-1, CHAR_PRNT_Y_POS-1, CHAR_INPT_Y_POS+1,  COLR_PRRED, COLR_BLACK);
    screen.draw_string("NOBLE OS", 0, 0, COLR_WHITE, COLR_BLACK);
    screen.draw_string("HYDROGEN BOOTLOADER", 0, CHAR_SCRN_X_DIM - 20 - CURRENT_VERSION.len(), COLR_WHITE, COLR_BLACK);
    screen.draw_string(CURRENT_VERSION, 0, CHAR_SCRN_X_DIM - CURRENT_VERSION.len(), COLR_WHITE, COLR_BLACK);
    screen.characterframe_render();
    writeln!(screen, "Welcome to Noble!");

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
                    *screen.print_y = CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP;
                    //Execute command and reset input stack
                    let command = &screen.input_as_chararray()[0..*screen.input_p].iter().collect::<String>();
                    let boot_command_return = command_processor(&mut screen, &system_table_boot, command);
                    screen.input_flush(Character::new(' ', COLR_WHITE, COLR_BLACK));
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
                    screen.character_input_draw_render(Character::new(input_char, COLR_WHITE, COLR_BLACK));
                }
            }
            //Modifier or Control Key
            Key::Special(scancode) =>{
                if scancode == ScanCode::UP && *screen.print_y > 0 {
                    *screen.print_y -= 1;
                    screen.printbuffer_draw_render();
                }
                else if scancode == ScanCode::DOWN && *screen.print_y < CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP{
                    *screen.print_y += 1;
                    screen.printbuffer_draw_render();
                }
                else if scancode == ScanCode::END{
                    *screen.print_y = CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP;
                    screen.printbuffer_draw_render();
                }
                else if scancode == ScanCode::PAGE_UP{
                    *screen.print_y = if *screen.print_y > CHAR_PRNT_Y_DIM_DSP {*screen.print_y - CHAR_PRNT_Y_DIM_DSP} else {0};
                    screen.printbuffer_draw_render();
                }
                else if scancode == ScanCode::PAGE_DOWN{
                    *screen.print_y = if *screen.print_y < CHAR_PRNT_Y_DIM_MEM - 2*CHAR_PRNT_Y_DIM_DSP {*screen.print_y + CHAR_PRNT_Y_DIM_DSP} else {CHAR_PRNT_Y_DIM_MEM - CHAR_PRNT_Y_DIM_DSP};
                    screen.printbuffer_draw_render();
                }
            }
        }
    }
    writeln!(screen, "Simple REPL exited.");

    // LOAD KERNEL
    //Find kernel on disk
    let mut fs_root = simple_file_system.open_volume().expect_success("File system root failed to open.");
    let fs_kernel_handle = fs_root.open("noble", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"noble\".").
        open("helium", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"helium\".").
        open("x86-64.elf", FileMode::Read, FileAttribute::empty()).expect_success("File system kernel open failed at \"x86-64.elf\".");
    let mut fs_kernel = unsafe { RegularFile::new(fs_kernel_handle) };
    writeln!(screen, "Found kernel on file system.");
    //Read ELF header
    let k_eheader = match ELFFileHeader64::new(&{let mut buf = [0u8;0x40]; fs_kernel.read(&mut buf).expect_success("Kernel file read failed at ELF header."); buf}){
        Ok(result) => result,
        Err(error) => panic!("{}", error)
    };
    //Check ELF header validity
    if k_eheader.ei_osabi != 0x00 {writeln!(screen, "Kernel load: Incorrect ei_osapi."); panic!();}
    if k_eheader.ei_abiversion != 0x00 {writeln!(screen, "Kernel load: Incorrect ei_abiversion."); panic!();}
    if k_eheader.e_machine != 0x3E {writeln!(screen, "Kernel load: Incorrect e_machine."); panic!();}
    //Print ELF header info
    writeln!(screen, "Kernel Entry Point:                 0x{:04X}", k_eheader.e_entry);
    writeln!(screen, "Kernel Program Header Offset:       0x{:04X}", k_eheader.e_phoff);
    writeln!(screen, "Kernel Section Header Offset:       0x{:04X}", k_eheader.e_shoff);
    writeln!(screen, "Kernel ELF Header Size:             0x{:04X}", k_eheader.e_ehsize);
    writeln!(screen, "Kernel Program Header Entry Size:   0x{:04X}", k_eheader.e_phentsize);
    writeln!(screen, "Kernel Program Header Number:       0x{:04X}", k_eheader.e_phnum);
    writeln!(screen, "Kernel Section Header Entry Size:   0x{:04X}", k_eheader.e_shentsize);
    writeln!(screen, "Kernel Section Header Number:       0x{:04X}", k_eheader.e_shnum);
    writeln!(screen, "Kernel Section Header String Index: 0x{:04X}", k_eheader.e_shstrndx);
    //Read program headers
    let mut code_list:Vec<(u64,u64,u64,u64)> = vec![(0,0,0,0);5];
    let mut code_num = 0;
    let mut kernel_size = 0;
    for _ in 0..k_eheader.e_phnum{
        let pheader = match ELFProgramHeader64::new(&{let mut buf = [0u8;0x38]; fs_kernel.read(&mut buf).expect_success("Kernel file read failed at program header."); buf}, k_eheader.ei_data){
            Ok(result) => result,
            Err(error) => panic!("{}", error)
        };
        if pheader.p_type == 0x01 {
            code_list[code_num] = (pheader.p_offset, pheader.p_filesz, pheader.p_vaddr, pheader.p_memsz);
            code_num = code_num + 1;
            if pheader.p_vaddr + pheader.p_memsz > kernel_size {
                kernel_size = pheader.p_vaddr + pheader.p_memsz;
            }
        }
    }
    writeln!(screen, "Kernel Code Size: 0x{:X}", kernel_size);
    //Allocate memory for code
    let code_pointer = unsafe { allocate_memory(boot_services, kernel_size as usize, EFI_PAGE_SIZE as usize) };
    for i in 0..code_num{
        fs_kernel.set_position(code_list[i].0).expect_success("Kernel file read failed at seeking code.");
        let mut buf:Vec<u8> = vec![0; code_list[i].1 as usize];
        fs_kernel.read(&mut buf).expect_success("Kernel file read failed at loading code.");
        unsafe { copy_nonoverlapping(buf.as_ptr(), code_pointer.add(code_list[i].2 as usize), code_list[i].3 as usize); }
    }

    // TODO SWITCH MEMORY MAPS
    //Top level locations
    let frame_oct = 0o775;
    let _kernel_oct = 0o776;
    let table_oct = 0o777;
    unsafe{
        //4th level page table to be placed into CR3
        let pml4:*mut u8 = allocate_memory(boot_services, EFI_PAGE_SIZE as usize, EFI_PAGE_SIZE as usize);
        write_pte(pml4.add(table_oct), pml4 as u64);
        //TODO Kernel Page Table
        //TODO Frame Buffer Page Table
        let pdpte_frame = create_pdpte_offset(boot_services, graphics_frame_pointer as u64, PIXL_SCRN_Y_DIM*PIXL_SCRN_X_DIM*PIXL_SCRN_B_DEP);
        write_pte(pml4.add(frame_oct), pdpte_frame as u64);
        writeln!(screen, "Frame Buffer PDPTE Location: {:p}", pdpte_frame);
    }

    // BOOT SEQUENCE
    //Exit Boot Services
    let mut memory_map_buffer = [0; 10000];
    let (_table_runtime, _esi) = system_table_boot.exit_boot_services(_handle, &mut memory_map_buffer).expect_success("Boot services exit failed");
    exit_boot_services();
    //Declare boot services exited
    writeln!(screen, "Boot Services exited.");
    //Kernel entry
    let kernel_entry_fn = unsafe { transmute::<*mut u8, extern "sysv64" fn(*mut u8) -> !>(code_pointer.add(k_eheader.e_entry as usize)) };
    kernel_entry_fn(graphics_frame_pointer);

    // HALT COMPUTER
    //writeln!(screen, "Halt reached.");
    //unsafe { asm!("HLT"); }
    //loop{}
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
//Evaluates and executes a bootloader command and returns a code
fn command_processor(screen: &mut Screen, system_table: &SystemTable<Boot>, command: &str) -> u8 {
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
fn command_mem(screen: &mut Screen, boot_service: &BootServices){
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
        let size = size_pages * EFI_PAGE_SIZE;
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
fn command_peek(screen: &mut Screen, command: &str) {
    //Read subcommand from arguments
    let split:Vec<&str> = command.split(" ").collect();
    //Handle if number of arguments is incorrect
    if split.len() != 4 {
        writeln!(screen, "Incorrect number of arguments: {} (requires 3).", split.len());
        return;
    }
    //Read numbers from argument
    let address = match usize::from_str_radix(split[2], 16){
        Ok(i) => {i},
        Err(_) => {
            //Handle if address is not a valid number
            writeln!(screen, "2nd argument not valid: {}", split[2]);
            return;
        }
    } as *mut u8;
    let size = match split[3].to_string().parse::<usize>(){
        Ok(i) => {i},
        Err(_) => {
            //Handle if length is not a valid number
            writeln!(screen, "3rd argument not valid: {}", split[3]);
            return;
        }
    };
    //Read memory
    if split[1].eq_ignore_ascii_case("u64b"){
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
        unsafe {
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                writeln!(screen, "0x{:016X}: 0o{:022o}", address as usize + i*8, num);
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
unsafe fn allocate_memory(boot_services: &BootServices, size: usize, align: usize) -> *mut u8 {
    let mem_ty = MemoryType::LOADER_CODE;
    if align > 8 {
        let pointer =
            if let Ok(pointer_from_services) = boot_services.allocate_pool(mem_ty, size + align).warning_as_error() {
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
        boot_services.allocate_pool(mem_ty, size).warning_as_error().unwrap_or(ptr::null_mut())
    }
}

//Write page table entry
unsafe fn write_pte(entry_location: *mut u8, address: u64) -> bool {
    //Return if invalid address
    if address | 0o7777 != 0 {return false;}
    //Convert to bytes
    let bytes = u64::to_le_bytes(address);
    //Write address
    for i in 0..8{
        write_volatile(entry_location.add(i), bytes[i]);
    }
    //Write flags
    write_volatile(entry_location, 0b00000001);
    return true;
}

//Reserve memory area as page directory pointer table mapped to an offset region of physical memory
unsafe fn create_pdpte_offset(boot_services:&BootServices, offset: u64, size: usize) -> *mut u8{
    //New level 3 table
    let pdpte = allocate_memory(boot_services, EFI_PAGE_SIZE as usize, EFI_PAGE_SIZE as usize);
    let mut pde: *mut u8 = 0 as *mut u8;
    let mut pte: *mut u8 = 0 as *mut u8;
    let pages = size/0b0000000000000000001000000000000 + if size%0b0000000000000000001000000000000 != 0 {1} else {0};
    for i in 0..pages{
        if i%0x100 == 0 {
            if i%0x20000 == 0 {
                //New level 2 table
                pde = allocate_memory(boot_services, EFI_PAGE_SIZE as usize, EFI_PAGE_SIZE as usize);
                write_pte(pdpte.add(i/0x20000), pde as u64);
            }
            //New level 1 table
            pte = allocate_memory(boot_services, EFI_PAGE_SIZE as usize, EFI_PAGE_SIZE as usize);
            write_pte(pde.add(i/0x100), pte as u64);
        }
        //New page
        write_pte(pte.add(i%0x100), offset + (i*0x100) as u64);
    }
    return pdpte;
}


// STRUCTS
//64-bit ELF File Header
#[derive(Debug)]
struct ELFFileHeader64 {
    ei_magic:      [u8;4],
    ei_class:      u8,
    ei_data:       u8,
    ei_version:    u8,
    ei_osabi:      u8,
    ei_abiversion: u8,
    ei_pad:        [u8;7],
    e_type:        u16,
    e_machine:     u16,
    e_version:     u32,
    e_entry:       u64,
    e_phoff:       u64,
    e_shoff:       u64,
    e_flags:       [u8;4],
    e_ehsize:      u16,
    e_phentsize:   u16,
    e_phnum:       u16,
    e_shentsize:   u16,
    e_shnum:       u16,
    e_shstrndx:    u16
}
impl ELFFileHeader64 {
    // CONSTRUCTOR
    pub fn new(head: &[u8;0x40]) -> Result<ELFFileHeader64, &str> {
        //Check ei_magic
        if head[0x00..0x04] != [0x7Fu8, 0x45u8, 0x4cu8, 0x46u8]{return Result::Err("ELFHeader64: Invalid ei_magic (magic number).")}
        //Check ei_class
        if head[0x04] != 0x02 {return Result::Err("ELFHeader64: Invalid ei_class (bit width).")}
        //Check ei_data
        let (u16_fb, u32_fb, u64_fb):(fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match head[0x05]{
            0x01 => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
            0x02 => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
            _ => {return Result::Err("ELFHeader64: Invalid ei_data (endianness).")}
        };
        //Check ei_version
        if head[0x06] != 0x01 {return Result::Err("ELFHeader64: Invalid ei_version (ELF version).")}
        //Check ei_pad
        if head[0x09..0x10] != [0x00u8; 7] {return Result::Err("ELFHeader64: Invalid ei_pad (padding).")}
        //Check e_version
        if u32_fb(head[0x14..0x18].try_into().unwrap()) != 0x01 {return Result::Err("ELFHeader64: Invalid e_version (ELF version).")}
        //Check e_phoff
        if u64_fb(head[0x20..0x28].try_into().unwrap()) != 0x40 {return Result::Err("ELFHeader64: Invalid e_phoff (program header offset).")}
        //Check e_ehsize
        if u16_fb(head[0x34..0x36].try_into().unwrap()) != 0x40 {return Result::Err("ELFHeader64: Invalid e_ehsize (ELF header size).")}
        //Check e_phentsize
        if u16_fb(head[0x36..0x38].try_into().unwrap()) != 0x38 {return Result::Err("ELFHeader64: Invalid e_phentsize (program header entry size).")}
        //Check e_shentsize
        if u16_fb(head[0x3A..0x3C].try_into().unwrap()) != 0x40 {return Result::Err("ELFHeader64: Invalid e_shentsize (section header entry size).")}
        //Check e_shstrndx is not larger than e_shnum
        if u16_fb(head[0x3E..0x40].try_into().unwrap()) >= u16_fb(head[0x3C..0x3E].try_into().unwrap()) {return Result::Err("Invalid e_shstrndx (section header strings index) according to e_shnum (section header number).")}
        //Return
        return Result::Ok(ELFFileHeader64 {
            ei_magic:             head[0x00..0x04].try_into().unwrap(),
            ei_class:             head[0x04],
            ei_data:              head[0x05],
            ei_version:           head[0x06],
            ei_osabi:             head[0x07],
            ei_abiversion:        head[0x08],
            ei_pad:               head[0x09..0x10].try_into().unwrap(),
            e_type:        u16_fb(head[0x10..0x12].try_into().unwrap()),
            e_machine:     u16_fb(head[0x12..0x14].try_into().unwrap()),
            e_version:     u32_fb(head[0x14..0x18].try_into().unwrap()),
            e_entry:       u64_fb(head[0x18..0x20].try_into().unwrap()),
            e_phoff:       u64_fb(head[0x20..0x28].try_into().unwrap()),
            e_shoff:       u64_fb(head[0x28..0x30].try_into().unwrap()),
            e_flags:              head[0x30..0x34].try_into().unwrap(),
            e_ehsize:      u16_fb(head[0x34..0x36].try_into().unwrap()),
            e_phentsize:   u16_fb(head[0x36..0x38].try_into().unwrap()),
            e_phnum:       u16_fb(head[0x38..0x3A].try_into().unwrap()),
            e_shentsize:   u16_fb(head[0x3A..0x3C].try_into().unwrap()),
            e_shnum:       u16_fb(head[0x3C..0x3E].try_into().unwrap()),
            e_shstrndx:    u16_fb(head[0x3E..0x40].try_into().unwrap())
        })
    }
}

//64-bit ELF Program Header
#[derive(Debug)]
struct ELFProgramHeader64 {
    p_type:   u32,
    p_flags:  [u8;4],
    p_offset: u64,
    p_vaddr:  u64,
    p_paddr:  u64,
    p_filesz: u64,
    p_memsz:  u64,
    p_align:  u64
}
impl ELFProgramHeader64 {
    // CONSTRUCTOR
    pub fn new(head: &[u8;0x38], endianness: u8) -> Result<ELFProgramHeader64, &str>{
        let (_u16_fb, u32_fb, u64_fb):(fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness{
            0x01 => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
            0x02 => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
            _ => {return Result::Err("ELFHeader64: Invalid endianness.")}
        };
        return Result::Ok(ELFProgramHeader64 {
            p_type:   u32_fb(head[0x00..0x04].try_into().unwrap()),
            p_flags:         head[0x04..0x08].try_into().unwrap(),
            p_offset: u64_fb(head[0x08..0x10].try_into().unwrap()),
            p_vaddr:  u64_fb(head[0x10..0x18].try_into().unwrap()),
            p_paddr:  u64_fb(head[0x18..0x20].try_into().unwrap()),
            p_filesz: u64_fb(head[0x20..0x28].try_into().unwrap()),
            p_memsz:  u64_fb(head[0x28..0x30].try_into().unwrap()),
            p_align:  u64_fb(head[0x30..0x38].try_into().unwrap())
        })
    }
}
