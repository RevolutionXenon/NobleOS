#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(box_syntax)]
#![feature(asm)]
#![feature(bench_black_box)]

extern crate rlibc;
extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use photon::line_draw::draw_hline_to_textframe;
use photon::line_draw::draw_vline_to_textframe;
use x86_64::registers::control::*;
use core::convert::TryInto;
use core::intrinsics::copy_nonoverlapping;
use core::mem::transmute;
use core::ptr;
use core::ptr::read_volatile;
use core::ptr::write_volatile;
use uefi::alloc::exit_boot_services;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::{Input, Key, ScanCode};
use uefi::proto::media::file::{File, FileAttribute, FileMode, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::MemoryType;
use uefi::table::runtime::ResetType;
use photon::*;

mod command;

const EFI_PAGE_SIZE: u64 = 0x1000; //MEMORY PAGE SIZE (4KiB)
const CURRENT_VERSION: &str = "v2021-08-03";


// MAIN
//Main entry point after firmware boot
#[entry]
fn efi_main(_handle: Handle, system_table_boot: SystemTable<Boot>) -> Status {
    // INITIALIZATION
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
    let mut screen_framebuffer:Box<[u8;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP]> = box[0;PIXL_SCRN_X_DIM*PIXL_SCRN_Y_DIM*PIXL_SCRN_B_DEP]; //Framebuffer for double buffering Screen
    let mut screen_charbuffer: [char;CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM] = [' ';CHAR_SCRN_X_DIM*CHAR_SCRN_Y_DIM];                                     //Text data of Screen
    //Print Result Window variables
    let mut print_xbuffer: usize = 0;                                                                                                              //Position of "print head" of Print Result Window
    let mut print_charbuffer: [char; CHAR_PRNT_X_DIM * CHAR_PRNT_Y_DIM_MEM]=[' '; CHAR_PRNT_X_DIM * CHAR_PRNT_Y_DIM_MEM];                          //Text data of Print Result Window
    //Input Window variables
    let mut input_pbuffer: usize = 0;                                                                                                              //Position of "print head" of Input Window
    let mut input_charstack: [char; CHAR_INPT_X_DIM * CHAR_INPT_Y_DIM_MEM] = [' '; CHAR_INPT_X_DIM * CHAR_INPT_Y_DIM_MEM];                         //Text data of Input Window
    //Graphics Output Protocol set graphics mode
    set_graphics_mode(graphics_output_protocol);
    let _st = graphics_output_protocol.current_mode_info().stride();
    let _pf = graphics_output_protocol.current_mode_info().pixel_format();
    let _s = graphics_output_protocol.frame_buffer().size();
    //Draw first screen
    draw_color_to_pixelframe(&mut screen_framebuffer, COLR_BACK);
    draw_pixelframe_to_hardwarebuffer(graphics_frame_pointer, &screen_framebuffer);
    //UI setup
    draw_textframe_to_pixelframe(&mut screen_framebuffer, &screen_charbuffer, COLR_BACK, COLR_FORE);
    draw_pixelframe_to_hardwarebuffer(graphics_frame_pointer, &screen_framebuffer);
    //Wait for 2 Seconds
    system_table_boot.boot_services().stall(2_000_000);

    // PRINT SETUP
    //Position variable
    let mut vertical_positon = CHAR_PRNT_Y_DIM_MEM;
    //Print Macros
    macro_rules! print { ($text:expr) => {
        print_str_to_textbuffer(&mut print_charbuffer, CHAR_PRNT_X_DIM, CHAR_PRNT_Y_DIM_MEM, &mut print_xbuffer, $text);
        for y in 0..CHAR_PRNT_Y_DIM_DSP {
            let s = (CHAR_PRNT_Y_POS+y)*CHAR_SCRN_X_DIM+CHAR_PRNT_X_POS;
            let p = (vertical_positon-CHAR_PRNT_Y_DIM_DSP+y)*CHAR_PRNT_X_DIM;
            screen_charbuffer[s..s+CHAR_PRNT_X_DIM].copy_from_slice(&print_charbuffer[p..p+CHAR_PRNT_X_DIM]);
        }
        draw_textframe_to_pixelframe(&mut screen_framebuffer, &screen_charbuffer, COLR_BACK, COLR_FORE);
        draw_pixelframe_to_hardwarebuffer(graphics_frame_pointer, &screen_framebuffer);
    };}
    macro_rules! println { ($text: expr) => {
        print!($text);print!("\n");
    };}
    //Print Startup
    draw_hline_to_textframe(&mut screen_charbuffer, CHAR_PRNT_Y_POS-1, 0, CHAR_SCRN_X_DIM-1);
    draw_hline_to_textframe(&mut screen_charbuffer, CHAR_INPT_Y_POS-1, 0, CHAR_SCRN_X_DIM-1);
    draw_hline_to_textframe(&mut screen_charbuffer, CHAR_INPT_Y_POS+1, 0, CHAR_SCRN_X_DIM-1);
    draw_vline_to_textframe(&mut screen_charbuffer, 0, CHAR_PRNT_Y_POS-1, CHAR_INPT_Y_POS+1);
    draw_vline_to_textframe(&mut screen_charbuffer, CHAR_SCRN_X_DIM-1, CHAR_PRNT_Y_POS-1, CHAR_INPT_Y_POS+1);
    println!("Welcome to Noble!");
    print!("Hydrogen Bootloader "); println!(CURRENT_VERSION);

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
                    vertical_positon = CHAR_PRNT_Y_DIM_MEM;
                    //Execute command and get return value
                    let boot_command_return = boot_commands(&system_table_boot, &input_charstack[0..input_pbuffer].iter().collect::<String>());
                    print!(&boot_command_return.1);
                    input_pbuffer = 0;
                    input_charstack = [' '; CHAR_INPT_X_DIM * CHAR_INPT_Y_DIM_MEM];
                    //Refresh Screen
                    screen_charbuffer[(CHAR_SCRN_X_DIM *(CHAR_SCRN_Y_DIM -2))+1..(CHAR_SCRN_X_DIM *(CHAR_SCRN_Y_DIM -2))+1+ CHAR_INPT_X_DIM].clone_from_slice(&input_charstack[0..CHAR_INPT_X_DIM]);
                    draw_textframe_to_pixelframe(&mut screen_framebuffer, &screen_charbuffer, COLR_BACK, COLR_FORE);
                    draw_pixelframe_to_hardwarebuffer(graphics_frame_pointer, &screen_framebuffer);
                    //Check Return Code
                    if boot_command_return.0 != 0 {
                        //Boot Sequence
                        if boot_command_return.0 == 1 {
                            //Exit to boot
                            break;
                        }
                        //Shutdown sequence
                        else if boot_command_return.0 == 2 {
                            //Wait for 5 seconds
                            system_table_boot.boot_services().stall(5000000);
                            //Shut down computer
                            system_table_boot.runtime_services().reset(ResetType::Shutdown, Status(0), None);
                        }
                    }
                }
                //User has typed a character
                else {
                    input_character(graphics_frame_pointer, &mut screen_framebuffer, &mut screen_charbuffer, &mut input_charstack, &mut input_pbuffer, input_char);
                }
            }
            //Modifier or Control Key
            Key::Special(scancode) =>{
                if scancode == ScanCode::UP && vertical_positon > CHAR_PRNT_Y_DIM_DSP {
                    vertical_positon -= 1;
                }
                else if scancode == ScanCode::DOWN && vertical_positon < CHAR_PRNT_Y_DIM_MEM{
                    vertical_positon += 1;
                }
                else if scancode == ScanCode::RIGHT{
                    print!("");
                }
                else if scancode == ScanCode::LEFT{
                    vertical_positon = CHAR_PRNT_Y_DIM_MEM;
                    print!("");
                }
            }
        }
    }
    println!("Simple REPL exited.");

    // FIND KERNEL ON DISK
    //Find kernel on disk
    let mut fs_root = simple_file_system.open_volume().expect_success("File system root failed to open.");
    let fs_kernel_handle = fs_root.open("noble", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"noble\".").
        open("helium", FileMode::Read, FileAttribute::DIRECTORY).expect_success("File system kernel open failed at \"helium\".").
        open("x86-64.elf", FileMode::Read, FileAttribute::empty()).expect_success("File system kernel open failed at \"x86-64.elf\".");
    let mut fs_kernel = unsafe { RegularFile::new(fs_kernel_handle) };
    println!("Found kernel on file system.");

    // LOAD ELF INFORMATION
    //Read ELF header
    let k_eheader = match ELFFileHeader64::new(&{let mut buf = [0u8;0x40]; fs_kernel.read(&mut buf).expect_success("Kernel file read failed at ELF header."); buf}){
        Ok(result) => result,
        Err(error) => panic!("{}", error)
    };
    //Check validity
    if k_eheader.ei_osabi != 0x00 {println!("Kernel load: Incorrect ei_osapi."); panic!();}
    if k_eheader.ei_abiversion != 0x00 {println!("Kernel load: Incorrect ei_abiversion."); panic!();}
    if k_eheader.e_machine != 0x3E {println!("Kernel load: Incorrect e_machine."); panic!();}
    //Print ELF header info
    println!(&format!("Kernel Entry Point:                 0x{:04X}", k_eheader.e_entry));
    println!(&format!("Kernel Program Header Offset:       0x{:04X}", k_eheader.e_phoff));
    println!(&format!("Kernel Section Header Offset:       0x{:04X}", k_eheader.e_shoff));
    println!(&format!("Kernel ELF Header Size:             0x{:04X}", k_eheader.e_ehsize));
    println!(&format!("Kernel Program Header Entry Size:   0x{:04X}", k_eheader.e_phentsize));
    println!(&format!("Kernel Program Header Number:       0x{:04X}", k_eheader.e_phnum));
    println!(&format!("Kernel Section Header Entry Size:   0x{:04X}", k_eheader.e_shentsize));
    println!(&format!("Kernel Section Header Number:       0x{:04X}", k_eheader.e_shnum));
    println!(&format!("Kernel Section Header String Index: 0x{:04X}", k_eheader.e_shstrndx));
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
    //Allocate memory for code
    let code_pointer = unsafe { reserve_code_space(boot_services, kernel_size as usize, EFI_PAGE_SIZE as usize) };
    for i in 0..code_num{
        fs_kernel.set_position(code_list[i].0).expect_success("Kernel file read failed at seeking code.");
        let mut buf:Vec<u8> = vec![0; code_list[i].1 as usize];
        fs_kernel.read(&mut buf).expect_success("Kernel file read failed at loading code.");
        unsafe { copy_nonoverlapping(buf.as_ptr(), code_pointer.add(code_list[i].2 as usize), code_list[i].3 as usize); }
    }

    // PRE EXIT TESTING GROUNDS
    //println!(&format!("{:16X}", graphics_output_protocol.frame_buffer().as_mut_ptr() as usize));
    //println!(&format!("GOP: {:016X}", graphics_output_protocol.frame_buffer().as_mut_ptr() as usize));
    //let arg = graphics_output_protocol.frame_buffer().as_mut_ptr() as usize;
    //println!(&format!("Graphics output pointer at: {:p}", graphics_frame_pointer));
    println!(&format!("Kernel Code Size: 0x{:X}", kernel_size));

    //TODO SWITCH MEMORY MAPS
    let frame_oct = 0o775;
    let kernel_oct = 0o776;
    let table_oct = 0o777;
    unsafe{
        //4th level page table to be placed into CR3
        let pml4:*mut u8 = reserve_code_space(boot_services, EFI_PAGE_SIZE as usize, EFI_PAGE_SIZE as usize);
        write_pte(pml4.add(table_oct), pml4 as u64);
        //TODO Kernel Page Table
        //TODO Frame Buffer Page Table
        let pdpte_frame = create_pdpte_offset(boot_services, graphics_frame_pointer as u64, PIXL_SCRN_Y_DIM*PIXL_SCRN_X_DIM*PIXL_SCRN_B_DEP);
        write_pte(pml4.add(frame_oct), pdpte_frame as u64);

        println!(&format!("Frame Buffer PDPTE Location: {:p}", pdpte_frame));
    }

    // EXIT BOOT SERVICES
    //exit
    let mut memory_map_buffer = [0; 10000];
    let (_table_runtime, _esi) = system_table_boot.exit_boot_services(_handle, &mut memory_map_buffer).expect_success("Boot services exit failed");
    exit_boot_services();
    //Declare boot services exited
    println!("Boot Services exited.");

    // BOOT SEQUENCE
    
    //Kernel entry
    let kernel_entry_fn = unsafe { transmute::<*mut u8, extern "sysv64" fn(*mut u8) -> !>(code_pointer.add(k_eheader.e_entry as usize)) };
    //unsafe { asm!("mov rdi, ${0}", in(reg) graphics_frame_pointer as usize); }
    //kernel_entry_fn(graphics_frame_pointer);

    //HALT COMPUTER
    println!("Reached halting function.");
    unsafe { asm!("HLT"); }
    loop{}
}

//Set a larger graphics mode
fn set_graphics_mode(gop: &mut GraphicsOutput) {
    // We know for sure QEMU has a 1024x768 mode.
    let mode = gop
        .modes()
        .map(|mode| mode.expect("Warnings encountered while querying mode"))
        .find(|mode| {
            let info = mode.info();
            info.resolution() == (PIXL_SCRN_X_DIM, PIXL_SCRN_Y_DIM)
        })
        .unwrap();

    gop.set_mode(&mode)
        .expect_success("Failed to set graphics mode");
}


// COMMAND PROCESSOR
//Evaluates and executes a bootloader command and returns a return code and associated string
fn boot_commands(system_table: &SystemTable<Boot>, command: &str) -> (u8, String) {
    if command.eq_ignore_ascii_case("time"){
        //Time
        return (
            0x00,
            format!(">{}\n{}\n", command, match system_table.runtime_services().get_time(){
                Ok(v) => {let v = v.log(); format!{"{}-{:02}-{:02} {:02}:{:02}:{:02} UTC", v.year(), v.month(), v.day(), v.hour(), v.minute(), v.second()}},
                Err(e) => format!("Command failed: {:?}", e)}
            )
        )
    }
    else if command.eq_ignore_ascii_case("mem"){
        //Memory Map
        return (
            0x00,
            format!(">{}\n{}", command, command_mem(system_table.boot_services()))
        )
    }
    else if command.eq_ignore_ascii_case("shutdown"){
        //Shutdown
        return (
            0x02, // SHUTDOWN RETURN CODE : 0x02
            format!(">{}\nShutdown sequence started.\n", command)
        )
    }
    else if command.eq_ignore_ascii_case("boot"){
        //Boot
        return (
            0x01, // BOOT RETURN CODE : 0x01
            format!(">{}\nBoot sequence started.\n", command)
        )
    }
    else if command.eq_ignore_ascii_case("crd"){
        //Display control registers
        return (
            0x00,
            format!(">{}\nCR0:  {:016x}\nCR2:  {:016x}\nCR3:  {:016x}\nCR4:  {:016x}\nEFER: {:016x}\n", command, 
            Cr0::read().bits(), 
            Cr2::read().as_u64(), 
            Cr3::read().0.start_address(), 
            Cr4::read().bits(),
            Efer::read().bits())
        )
    }
    else if command.starts_with("peek "){
        return (
            0x00,
            format!(">{}\n{}", command, command_peek(command))
        )
    }
    else{
        //No result
        return (
            0x00,
            format!(">{}\nCommand not entered properly.\n", command)
        )
    }
}

//Display memory map contents to console
fn command_mem(boot_service: &BootServices) -> String{
    let mut result = "".to_string();
    // Get the estimated map size
    let map_size = boot_service.memory_map_size();

    result = format!("{}Map size: {}\n", result, map_size);

    // Build a buffer big enough to handle the memory map
    let mut buffer = vec![0;map_size+512];
    result = format!("{}Buffer len: {}\n", result, buffer.len());
    result = format!("{}Buffer cap: {}\n", result, buffer.capacity());

    let (_k, description_iterator) = boot_service
        .memory_map(&mut buffer)
        .expect_success("UEFI Memory Map retrieval failed.");

    let descriptors = description_iterator.copied().collect::<Vec<_>>();

    if descriptors.is_empty() {return "UEFI Memory Map is empty.\n".to_string();}

    // Print out a list of all the usable memory we see in the memory map.
    // Don't print out everything, the memory map is probably pretty big
    // (e.g. OVMF under QEMU returns a map with nearly 50 entries here).

    result = format!("{}Usable ranges: {}\n", result, descriptors.len());

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
        result = format!("{}{}: {:016x}-{:016x} ({:8}KiB / {:8}Pg)\n", result, memory_type_text, descriptor.phys_start, end_address, size/1024, size_pages);
    }
    return result;
}

//Display the raw contents of a part of memory
fn command_peek(command: &str) -> String {
    let mut result = "".to_string();
    let split:Vec<&str> = command.split(" ").collect();
    if split.len() != 4 {return "Incorrect number of arguments (requires 3).\n".to_string();}
    //Read subcommand from arguments
    //Read numbers from argument
    let address = match usize::from_str_radix(split[2], 16){
        Ok(i) => {i},
        Err(_) => {return format!("2nd argument not valid: {}\n", split[2]);}
    } as *mut u8;
    let size = match split[3].to_string().parse::<usize>(){
        Ok(i) => {i},
        Err(_) => {return format!("3rd argument not valid: {}\n", split[3]);}
    };
    //Read memory
    if split[1].eq_ignore_ascii_case("u64b"){
        unsafe{
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                result = format!("{}0b{:064b}\n", result, num);
            }
        }
        result = format!("{}{:p} {}\n", result, address, size);
        return result;
    }
    else if split[1].eq_ignore_ascii_case("u64x"){
        unsafe {
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                result = format!("{}0x{:016X}\n", result, num);
            }
        }
        result = format!("{}{:p} {}\n", result, address, size);
        return result;
    }
    else if split[1].eq_ignore_ascii_case("u64o"){
        unsafe {
            for i in 0..size {
                let mut le_bytes:[u8;8] = [0;8];
                for j in 0..8{asm!("mov {0}, [{1}]", out(reg_byte) le_bytes[j], in(reg) address.add(i*8 + j), options(readonly, nostack))}
                let num = u64::from_le_bytes(le_bytes);
                result = format!("{}0o{:022o}\n", result, num);
            }
        }
        result = format!("{}{:p} {}\n", result, address, size);
        return result;
    }
    return format!("1st argument not valid: {}\n", split[1]);
}


// MEMORY FUNCTIONS
//Allocate system memory
unsafe fn reserve_code_space(boot_services: &BootServices, size: usize, align: usize) -> *mut u8 {
    let mem_ty = MemoryType::LOADER_CODE;

    if align > 8 {
        let pointer =
            if let Ok(pointer_from_services) = boot_services.allocate_pool(mem_ty, size + align).warning_as_error(){
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
unsafe fn write_pte(entry_location: *mut u8, address: u64) -> bool{
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

//Reserve memory area as offset level 3 page table
unsafe fn create_pdpte_offset(boot_services:&BootServices, offset: u64, size: usize) -> *mut u8{
    //New level 3 table
    let pdpte = reserve_code_space(boot_services, EFI_PAGE_SIZE as usize, EFI_PAGE_SIZE as usize);
    let mut pde: *mut u8 = 0 as *mut u8;
    let mut pte: *mut u8 = 0 as *mut u8;
    let pages = size/0b0000000000000000001000000000000 + if size%0b0000000000000000001000000000000 != 0 {1} else {0};
    for i in 0..pages{
        if i%0x100 == 0 {
            if i%0x20000 == 0 {
                //New level 2 table
                pde = reserve_code_space(boot_services, EFI_PAGE_SIZE as usize, EFI_PAGE_SIZE as usize);
                write_pte(pdpte.add(i/0x20000), pde as u64);
            }
            //New level 1 table
            pte = reserve_code_space(boot_services, EFI_PAGE_SIZE as usize, EFI_PAGE_SIZE as usize);
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
    ei_magic:      [ u8;4],
    ei_class:        u8,
    ei_data:         u8,
    ei_version:      u8,
    ei_osabi:        u8,
    ei_abiversion:   u8,
    ei_pad:        [ u8;7],
    e_type:         u16,
    e_machine:      u16,
    e_version:      u32,
    e_entry:        u64,
    e_phoff:        u64,
    e_shoff:        u64,
    e_flags:       [ u8;4],
    e_ehsize:       u16,
    e_phentsize:    u16,
    e_phnum:        u16,
    e_shentsize:    u16,
    e_shnum:        u16,
    e_shstrndx:     u16
}
impl ELFFileHeader64 {
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
    p_type:    u32,
    p_flags:  [ u8;4],
    p_offset:  u64,
    p_vaddr:   u64,
    p_paddr:   u64,
    p_filesz:  u64,
    p_memsz:   u64,
    p_align:   u64
}
impl ELFProgramHeader64 {
    pub fn new(head: &[u8;0x38], endianness: u8) -> Result<ELFProgramHeader64, &str>{
        let (u16_fb, u32_fb, u64_fb):(fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness{
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