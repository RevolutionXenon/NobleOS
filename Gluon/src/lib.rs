// GLUON
// Gluon is the Noble boot information library:
// Memory locations of important objects
// Sizes and counts related to page tables

// HEADER
//Flags
#![no_std]

//Constants
pub const GLUON_VERSION:  &    str   = "v2021-08-07";                               //CURRENT VERSION OF GRAPHICS LIBRARY
//                                         SIGN PM4 PM3 PM2 PM1 OFFSET
pub const PHYSM_PHYS_OCT:      usize =        0o000usize;                           //PHYSICAL MEMORY PHYSICAL LOCATION PML4 OFFSET
pub const PHYSM_PHYS_PTR: *mut u8    = 0o000000_000_000_000_000_0000u64 as *mut u8; //PHYSICAL MEMORY PHYSICAL LOCATION POINTER
pub const FRAME_PHYS_PTR: *mut u8    = 0o000000_000_002_000_000_0000u64 as *mut u8; //FRAME BUFFER PHYSICAL LOCATION POINTER (QEMU OVMF SPECIFIC)
pub const KERNL_VIRT_OCT:      usize =        0o400usize;                           //KERNEL VIRTUAL LOCATION PML4 TABLE OFFSET
pub const KERNL_VIRT_PTR: *mut u8    = 0o177777_400_000_000_000_0000u64 as *mut u8; //KERNEL VIRTUAL LOCATION POINTER
pub const FRAME_VIRT_OCT:      usize =        0o775usize;                           //FRAME BUFFER VIRTUAL LOCATION PML4 OFFSET
pub const FRAME_VIRT_PTR: *mut u8    = 0o177777_775_000_000_000_0000u64 as *mut u8; //FRAME BUFFER VIRTUAL LOCATION POINTER
pub const PHYSM_VIRT_OCT:      usize =        0o776usize;                           //PHYSICAL MEMORY VIRTUAL LOCATION PML4 OFFSET
pub const PHYSM_VIRT_PTR: *mut u8    = 0o177777_776_000_000_000_0000u64 as *mut u8; //PHYSICAL MEMORY VIRTUAL LOCATION POINTER
pub const PGMAP_VIRT_OCT:      usize =        0o777usize;                           //PAGE MAP VIRTUAL LOCATION PML4 OFFSET
pub const PGMAP_VIRT_PTR: *mut u8    = 0o177777_777_000_000_000_0000u64 as *mut u8; //PAGE MAP VIRTUAL LOCATION POINTER
pub const PAGE_SIZE_4KIB:      usize =                      0o1_0000usize;          //MEMORY PAGE SIZE (4KiB)
pub const PAGE_SIZE_2MIB:      usize =                  0o1_000_0000usize;          //MEMORY PAGE SIZE (2MiB)
pub const PAGE_SIZE_1GIB:      usize =              0o1_000_000_0000usize;          //MEMORY PAGE SIZE (1GiB)
pub const PAGE_NMBR_LVL1:      usize =                        0o1000usize;          //NUMBER OF PAGE TABLE ENTRIES ONE LEVEL UP
pub const PAGE_NMBR_LVL2:      usize =                    0o100_0000usize;          //NUMBER OF PAGE TABLE ENTRIES TWO LEVELS UP
pub const PAGE_NMBR_LVL3:      usize =                0o100_000_0000usize;          //NUMBER OF PAGE TABLE ENTRIES THREE LEVELS UP
