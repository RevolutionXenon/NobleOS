// GLUON: x86-64
// Modules handling the x86-64 instruction set architecture:
//   instructions: Functions that shortcut intrinsic instructions from the x86-64 instruction set architecture
//   lapic:        Functions and objects related to the handling of the Local Advanced Programmable Interrupt Controller
//   msr:          Structs and objects handling Model Specific Registers
//   paging:       Structs, enums, and traits related to the contents and handling of x86-64 page tables
//   port:         Structs, functions, and traits related to the handling of ports
//   segmentation: Structs and enums related to the contents and handling of x86-64 GDT, IDT, and other segmentation structures
//   syscall:      Functions and Structs related to the handling of system calls on x86-64


// HEADER
//Modules
pub mod instructions;
pub mod lapic;
pub mod msr;
pub mod paging;
pub mod port;
pub mod registers;
pub mod segmentation;
pub mod syscall;
