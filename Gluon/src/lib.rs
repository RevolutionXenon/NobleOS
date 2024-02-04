//! GLUON
//! 
//! Gluon is the Noble architecture library:
//! * Instruction Set Architectures:
//!   * Modules handling the x86-64 instruction set architecture:
//!     * instructions:  Functions that shortcut intrinsic instructions from the x86-64 instruction set architecture
//!     * lapic:         Functions and objects related to the handling of the Local Advanced Programmable Interrupt Controller
//!     * msr:           Structs and objects handling Model Specific Registers
//!     * paging:        Structs, enums, and traits related to the contents and handling of x86-64 page tables
//!     * port:          Structs, functions, and traits related to the handling of ports
//!     * segmentation:  Structs and enums related to the contents and handling of x86-64 GDT, IDT, and other segmentation structures
//!     * syscall:       Functions and Structs related to the handling of system calls on x86-64
//! * System Architectures:
//!   * Modules handling the PC de-facto standard system architecture:
//!     * fat:           Structs and enums related to the contents and handling of the FAT16 file system
//!     * ports:         Functions and objects related to the handling of the PC architecture's standard port-space layout
//!     * pci:           Structs and objects related to the handling of the PCI bus
//!     * pic:           Functions related to the handling of the Programmable Interrupt Controller
//!     * pit:           Consts, Functions, and Enums related to the handling of the 8253 and 8254 Programmable Interval Timer
//!     * ps2:           Functions and objects related to the handling of the PS/2 controller and devices
//! * Operating System Architectures:
//!   * Modules handling the Unix System V operating system architecture:
//!     * executable:    Structs and enums related to the contents and handling of System V object files (ELF files)
//!   * Modules handling the Noble operating system architecture:
//!     * address_space: Constants related to the Noble address space layout
//!     * input_events:  Structs, enums, and functions for handling user keyboard, mouse, and controller inputs
//!     * file_system:   Structs and traits for handling file systems in a generic manner


// HEADER
//Flags
#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::inconsistent_digit_grouping)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![feature(naked_functions)]
#![feature(try_trait_v2)]

//Imports
use core::convert::TryFrom;

//Modules
pub mod noble;
pub mod pc;
pub mod sysv;
pub mod x86_64;

//Constants
pub const GLUON_VERSION: &str = "vDEV-2022"; //CURRENT VERSION OF LIBRARY


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
