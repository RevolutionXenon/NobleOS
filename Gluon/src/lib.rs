// GLUON
// Gluon is the Noble loading library:
// Constants and functions related to the Noble address space layout
// elf.rs: Structs, enums, and traits related to the contents and handling of ELF files
// idt.rs: Structs and enums related to the contents and handling of x86-64 GDT and IDT structures
// mem.rs: Structs, enums, and traits related to the contents and handling of x86-64 page tables
// pci.rs: Structs and objects related to the handling of the PCI bus
// pic.rs: Functions and objects related to the handling of the Programmable Interrupt Controller and Advanced Programmable Interrupt Controller
// ps2.rs: Functions and objects related to the handling of the PS/2 controller and devices


// HEADER
//Flags
#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::inconsistent_digit_grouping)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![feature(arbitrary_enum_discriminant)]
#![feature(asm)]

//Imports
pub mod elf;
pub mod idt;
pub mod mem;
pub mod pci;
pub mod pic;
pub mod ps2;
use core::convert::TryFrom;
use x86_64::instructions::port::*;

//Constants
pub const GLUON_VERSION: &str = "vDEV-2021-09-29"; //CURRENT VERSION OF GRAPHICS LIBRARY


// MACROS
//Numeric Enum
#[macro_export]
macro_rules!numeric_enum {(
        #[repr($repr:ident)]
        $(#[$a:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $value:expr,)*
        }
    ) => {
        #[repr($repr)]
        $(#[$a])*
        $vis enum $name {
            $($variant = $value,)*
        }
        impl TryFrom<$repr> for $name {
            type Error = ();
            fn try_from(from: $repr) -> Result<Self, ()> {
                match from {
                    $($value => Ok(Self::$variant),)*
                    _ => Err(())
                }
            }
        }
    }
}


// NOBLE ADDRESS SPACE
//Constants
//                                    SIGN PM5 PM4 PM3 PM2 PM1 OFFSET
pub const HIGHER_HALF_57:   usize = 0o_177_000_000_000_000_000_0000_usize; //HIGHER HALF SIGN EXTENSION IN FIVE LEVEL PAGE MAP (57-bit address space)
pub const HIGHER_HALF_48:   usize = 0o_177_777_000_000_000_000_0000_usize; //HIGHER HALF SIGN EXTENSION IN FOUR LEVEL PAGE MAP (48-bit address space)
pub const PHYSICAL_OCT:     usize = 0o_________000__________________usize; //PML4 OFFSET OF PHYSICAL MEMORY PHYSICAL LOCATION
pub const KERNEL_OCT:       usize = 0o_________400__________________usize; //PML4 OFFSET OF KERNEL VIRTUAL LOCATION
pub const PROGRAMS_OCT:     usize = 0o_________773__________________usize; //PML4 OFFSET OF PROGRAMS STORED BY BOOTLOADER
pub const FRAME_BUFFER_OCT: usize = 0o_________774__________________usize; //PML4 OFFSET OF SCREEN BUFFERS
pub const FREE_MEMORY_OCT:  usize = 0o_________775__________________usize; //PML4 OFFSET OF FREE PHYSICAL MEMORY VIRTUAL LOCATION
pub const IDENTITY_OCT:     usize = 0o_________776__________________usize; //PML4 OFFSET OF ALL PHYSICAL MEMORY VIRTUAL LOCATION
pub const PAGE_MAP_OCT:     usize = 0o_________777__________________usize; //PML4 OFFSET OF PAGE MAP VIRTUAL LOCATION
pub const PAGE_SIZE_4KIB:   usize = 0o_______________________1_0000_usize; //MEMORY PAGE SIZE (  4KiB), PAGE MAP LEVEL 1 ENTRY SIZE
pub const PAGE_SIZE_2MIB:   usize = 0o___________________1_000_0000_usize; //MEMORY PAGE SIZE (  2MiB), PAGE MAP LEVEL 2 ENTRY SIZE, PAGE MAP LEVEL 1 CAPACITY
pub const PAGE_SIZE_1GIB:   usize = 0o_______________1_000_000_0000_usize; //MEMORY PAGE SIZE (  1GiB), PAGE MAP LEVEL 3 ENTRY SIZE, PAGE MAP LEVEL 2 CAPACITY
pub const PAGE_SIZE_512G:   usize = 0o___________1_000_000_000_0000_usize; //MEMORY PAGE SIZE (512GiB),                              PAGE MAP LEVEL 3 CAPACITY
pub const PAGE_SIZE_256T:   usize = 0o_______1_000_000_000_000_0000_usize; //MEMORY PAGE SIZE (256TiB),                              PAGE MAP LEVEL 4 CAPACITY
pub const PAGE_SIZE_128P:   usize = 0o___1_000_000_000_000_000_0000_usize; //MEMORY PAGE SIZE (128PiB),                              PAGE MAP LEVEL 5 CAPACITY
pub const PAGE_NUMBER_1:    usize = 0o_________________________1000_usize; //NUMBER OF PAGE TABLE ENTRIES 1 LEVELS UP (               512)
pub const PAGE_NUMBER_2:    usize = 0o_____________________100_0000_usize; //NUMBER OF PAGE TABLE ENTRIES 2 LEVELS UP (           262,144)
pub const PAGE_NUMBER_3:    usize = 0o_________________100_000_0000_usize; //NUMBER OF PAGE TABLE ENTRIES 3 LEVELS UP (       134,217,728)
pub const PAGE_NUMBER_4:    usize = 0o_____________100_000_000_0000_usize; //NUMBER OF PAGE TABLE ENTRIES 4 LEVELS UP (    68,719,476,736)
pub const PAGE_NUMBER_5:    usize = 0o_________100_000_000_000_0000_usize; //NUMBER OF PAGE TABLE ENTRIES 5 LEVELS UP (35,184,372,088,832)
pub const KIB:              usize = 0o_________________________2000_usize; //ONE KIBIBYTE
pub const MIB:              usize = 0o_____________________400_0000_usize; //ONE MEBIBYTE
pub const GIB:              usize = 0o_______________1_000_000_0000_usize; //ONE GIBIBYTE
pub const TIB:              usize = 0o___________2_000_000_000_0000_usize; //ONE TEBIBYTE
pub const PIB:              usize = 0o_______4_000_000_000_000_0000_usize; //ONE PEBIBYTE

//Functions
pub fn oct_to_usize_4(pml4: usize, pml3: usize, pml2: usize, pml1: usize, offset: usize)   -> Result<usize,   &'static str> {
    if pml4   >= PAGE_NUMBER_1  {return Err("O4 to Pointer: PML4 oct out of bounds.")}
    if pml3   >= PAGE_NUMBER_1  {return Err("O4 to Pointer: PML3 oct out of bounds.")}
    if pml2   >= PAGE_NUMBER_1  {return Err("O4 to Pointer: PML2 oct out of bounds.")}
    if pml1   >= PAGE_NUMBER_1  {return Err("O4 to Pointer: PML1 oct out of bounds.")}
    if offset >= PAGE_SIZE_4KIB {return Err("O4 to Pointer: Offset out of bounds.")}
    let mut result: usize = if pml4 >= 0o400 {HIGHER_HALF_48} else {0};
    result |= pml4 << (0o14 + 0o11 + 0o11 + 0o11);
    result |= pml3 << (0o14 + 0o11 + 0o11);
    result |= pml2 << (0o14 + 0o11);
    result |= pml1 << (0o14);
    result |= offset;
    Ok(result)
}
pub fn oct_to_pointer_4(pml4: usize, pml3: usize, pml2: usize, pml1: usize, offset: usize) -> Result<*mut u8, &'static str> {
    Ok(oct_to_usize_4(pml4, pml3, pml2, pml1, offset)? as *mut u8)
}
pub fn oct4_to_usize(pml4: usize)                                                          -> Result<usize,   &'static str> {
    oct_to_usize_4(pml4, 0, 0, 0, 0)
}
pub fn oct4_to_pointer(pml4: usize)                                                        -> Result<*mut u8, &'static str> {
    oct_to_pointer_4(pml4, 0, 0, 0, 0)
}


// PC ARCHITECTURE
//Functions
pub fn io_wait() {
    unsafe {PORT_WAIT.write(0x00);}
}
/*pub fn cpuid(command: u32) -> (u32, u32, u32, u32) {
    let r1: u32;
    let r2: u32;
    let r3: u32;
    let r4: u32;
    unsafe {asm!(
        "PUSH RAX",
        "PUSH RBX",
        "PUSH RCX",
        "PUSH RDX",
        "MOV EAX, {command}",
        "CPUID",
        "MOV {r1}, EAX",
        "MOV {r2}, EBX",
        "MOV {r3}, ECX",
        "MOV {r4}, EDX",
        "POP RDX",
        "POP RCX",
        "POP RBX",
        "POP RAX",
        command = in(reg) command,
        r1 = out(reg) r1,
        r2 = out(reg) r2,
        r3 = out(reg) r3,
        r4 = out(reg) r4,
        options(nostack)
    )}
    (r1, r2, r3, r4)
}*/

//Ports
pub static mut PORT_PIC1_COMMAND:   PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x0020);
pub static mut PORT_PIC1_DATA:      PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0021);
pub static mut PORT_PIT_CHANNEL_1:  PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0040);
pub static mut PORT_PIT_CHANNEL_2:  PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0041);
pub static mut PORT_PIT_CHANNEL_3:  PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0042);
pub static mut PORT_PIT_COMMAND:    PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x0043);
pub static mut PORT_PS2_DATA:       PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0060);
pub static mut PORT_PS2_COMMAND:    PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x0064);
pub static mut PORT_PS2_STATUS:     PortGeneric<u8,  ReadOnlyAccess > = PortGeneric::<u8,  ReadOnlyAccess >::new(0x0064);
pub static mut PORT_WAIT:           PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x0080);
pub static mut PORT_PIC2_COMMAND:   PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x00A0);
pub static mut PORT_PIC2_DATA:      PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x00A1);
pub static mut PORT_PCI_INDEX:      PortGeneric<u32, ReadWriteAccess> = PortGeneric::<u32, ReadWriteAccess>::new(0x0CF8);
pub static mut PORT_PCI_DATA:       PortGeneric<u32, ReadWriteAccess> = PortGeneric::<u32, ReadWriteAccess>::new(0x0CFC);
