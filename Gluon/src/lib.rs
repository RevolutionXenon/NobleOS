// GLUON
// Gluon is the Noble architecture library:
// Modules handling the x86-64 CPU architecture:
//   lapic:         Functions and objects related to the handling of the Local Advanced Programmable Interrupt Controller
//   paging:        Structs, enums, traits, constants, and functions related to the contents and handling of x86-64 page tables
//   pci:           Structs and objects related to the handling of the PCI bus
//   pic:           Functions related to the handling of the Programmable Interrupt Controller
//   segmentation:  Structs and enums related to the contents and handling of x86-64 GDT, IDT, and other segmentation structures
//   syscall:       Structs and functions related to the handling of system calls on x86-64
// Modules handling the PC de-facto standard system architecture:
//   fat:           Structs and enums related to the contents and handling of the FAT16 file system
//   ports:         Functions and objects related to the handling of the PC architecture's standard port-space layout
//   ps2:           Functions and objects related to the handling of the PS/2 controller and devices
// Modules handling the Unix System V OS architecture:
//   executable:    Structs and enums related to the contents and handling of System V object files (ELF files)
// Modules handling the Noble OS architecture:
//   address_space: Constants related to the Noble address space layout
//   input_events:  Structs, enums, and functions for handling user keyboard, mouse, and controller inputs
//   file_system:   Structs and traits for handling file systems in a generic manner


// HEADER
//Flags
#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::inconsistent_digit_grouping)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![feature(arbitrary_enum_discriminant)]
#![feature(asm_sym)]
#![feature(naked_functions)]

//Imports
use core::convert::TryFrom;

//Modules
pub mod noble;
pub mod pc;
pub mod sysv;
pub mod x86_64;

//Constants
pub const GLUON_VERSION: &str = "vDEV-2022-02-10"; //CURRENT VERSION OF LIBRARY


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
