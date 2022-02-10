// GLUON: x86-64
// Modules handling the x86-64 CPU architecture:
//   lapic:        Functions and objects related to the handling of the Local Advanced Programmable Interrupt Controller
//   paging:       Structs, enums, and traits related to the contents and handling of x86-64 page tables
//   pci:          Structs and objects related to the handling of the PCI bus
//   pic:          Functions related to the handling of the Programmable Interrupt Controller
//   segmentation: Structs and enums related to the contents and handling of x86-64 GDT, IDT, and other segmentation structures
//   syscall:      Functions and Structs related to the handling of system calls on x86-64


// HEADER
//Modules
pub mod lapic;
pub mod paging;
pub mod pci;
pub mod pic;
pub mod segmentation;
pub mod syscall;
