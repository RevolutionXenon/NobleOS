# Noble Operating System

<img src="./.materials/logo-01.png" alt="Noble Logo (Version 1)" width="300"/>

**Noble is a lightweight microkernel and IPC based operating system built with Rust which is not a clone of any existing operating system.**

**Noble is currently a work in progress in its earliest stages.**

# Components

## Hydrogen Bootloader

A UEFI stub which handles:

* Memory and control register diagnostics
* Virtual memory initialization
* Kernel entry
* (PLANNED) Kernel space binary loading

## Helium Kernel

An ELF binary which handles:

* Code execution
* Interrupt handling
* CPU time sharing
* (PLANNED) System call handling
* (PLANNED) Thread management
* (PLANNED) Program loading
* (PLANNED) Inter-process communication handling

## Photon Graphics Library

A Rust Library which handles:

* Drawing text to a frame buffer

## Gluon Architecture Library

A Rust Library which handles:

* The Noble address space layout
* x86-64:
    * Long mode page tables
    * Segmentation data structures
    * The syscall instruction
    * The PCI bus
    * The PS/2 controller and devices
    * The Programmable Interrupt Controller and Advanced Programmable Interrupt Controller
* System V:
    * System V object files (ELF files)