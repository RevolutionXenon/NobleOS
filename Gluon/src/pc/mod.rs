// GLUON: PC
// Modules handling the PC de-facto standard system architecture:
//   fat:   Structs and enums related to the contents and handling of the FAT16 file system
//   ports: Functions and objects related to the handling of the PC architecture's standard port-space layout
//   pci:   Structs and objects related to the handling of the PCI bus
//   pic:   Functions related to the handling of the Programmable Interrupt Controller
//   pit:   Consts, Functions, and Enums related to the handling of the 8253 and 8254 Programmable Interval Timer
//   ps2:   Functions and objects related to the handling of the PS/2 controller and devices


// HEADER
//Modules
pub mod fat;
pub mod ports;
pub mod pci;
pub mod pic;
pub mod pit;
pub mod ps2;
