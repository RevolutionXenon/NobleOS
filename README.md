# Noble Operating System

<img src="./.materials/logo-01.png" alt="Noble Logo (Version 1)" width="300"/>

**Noble is a lightweight microkernel and IPC based operating system built with Rust which is not a clone of any existing operating system.**

**Noble is currently a work in progress in its earliest stages.**

# Components

## Hydrogen Bootloader

A UEFI stub which handles:

* Memory and Control Register Diagnostics
* Virtual Memory Initialization
* Kernel Entry
* (PLANNED) Kernel Space Binary Loading

## Helium Kernel

An ELF binary which handles:

* Code Execution
* Interrupt Handling
* CPU Time Sharing
* (PLANNED) System Call Handling
* (PLANNED) Thread Management
* (PLANNED) Program Loading
* (PLANNED) Inter-Process Communication Handling

## Photon Graphics Library

A Rust Library which handles:

* Drawing Text

## Gluon Architecture Library

A Rust Library which handles:

* The x86-64 Instruction Set Architecture:
  * Intrinsic Instructions
  * The Local Advanced Programmable Interrupt Controller
  * Model Specific Registers
  * Long Mode Page Tables
  * Segmentation Data Structures
  * System Calls
* The PC Defacto Standard System Architecture
  * The File Allocation Table (FAT) File System
  * The PC's Standard I/O-space Layout
  * The PCI Bus
  * The 8259 Programmable Interrupt Controller
  * The 8253 and 8254 Programmable Interval Timer
  * The 8042 PS/2 Controller and Devices
* The System V OS Architecture:
  * System V Object Files (ELF Files)
* The Noble OS Architecture:
  * The Noble Address Space Layout
  * User Keyboard, Mouse, and Controller Inputs
  * Noble File System Handles
