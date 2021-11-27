# Noble Operating System

![logo](./.materials/logo.png)

**Noble is a microkernel and IPC based operating system which runs on the Helium kernel.**

**Noble is a work in progress in its early stages.**

# Components

## Hydrogen Bootloader

A UEFI stub which handles:

* Memory and control register diagnostics
* Virtual memory initialization
* Kernel booting
* Kernel space binary loading (PLANNED)

## Helium Kernel

An ELF Binary which handles:

* Thread management
* Code execution
* CPU time sharing
* Interrupt handling
* System call handling
* Program loading (PLANNED)
* Pipe and shared memory management (PLANNED)

## Photon Graphics Library

A Rust Library which handles:

* Drawing text to a frame buffer

## Gluon Architecture Library

A Rust Library which handles:

* The Noble address space layout
* The contents of ELF files
* x86-64:
    * The GDT and IDT structures
    * The structure of page tables
    * The PCI bus
    * The Programmable Interrupt Controller and Advanced Programmable Interrupt Controller
    * The PS/2 controller and devices