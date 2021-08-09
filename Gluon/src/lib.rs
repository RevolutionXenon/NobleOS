// GLUON
// Gluon is the Noble boot information library:
// Memory locations of important objects
// Sizes and counts related to page tables

// HEADER
//Flags
#![no_std]

//Constants
pub const GLUON_VERSION:  &    str   = "vDEV-2021-08-09";                             //CURRENT VERSION OF GRAPHICS LIBRARY
//                                          SIGN PM4 PM3 PM2 PM1 OFFSET
pub const PHYSM_PHYS_OCT:      usize = 0o________000__________________usize;          //PHYSICAL MEMORY PHYSICAL LOCATION PML4 OFFSET
pub const PHYSM_PHYS_PTR: *mut u8    = 0o_000000_000_000_000_000_0000_u64 as *mut u8; //PHYSICAL MEMORY PHYSICAL LOCATION POINTER
pub const KERNL_VIRT_OCT:      usize = 0o________400__________________usize;          //KERNEL VIRTUAL LOCATION PML4 TABLE OFFSET
pub const KERNL_VIRT_PTR: *mut u8    = 0o_177777_400_000_000_000_0000_u64 as *mut u8; //KERNEL VIRTUAL LOCATION POINTER
pub const FRAME_VIRT_OCT:      usize = 0o________775__________________usize;          //FRAME BUFFER VIRTUAL LOCATION PML4 OFFSET
pub const FRAME_VIRT_PTR: *mut u8    = 0o_177777_775_000_000_000_0000_u64 as *mut u8; //FRAME BUFFER VIRTUAL LOCATION POINTER
pub const PHYSM_VIRT_OCT:      usize = 0o________776__________________usize;          //PHYSICAL MEMORY VIRTUAL LOCATION PML4 OFFSET
pub const PHYSM_VIRT_PTR: *mut u8    = 0o_177777_776_000_000_000_0000_u64 as *mut u8; //PHYSICAL MEMORY VIRTUAL LOCATION POINTER
pub const PGMAP_VIRT_OCT:      usize = 0o________777__________________usize;          //PAGE MAP VIRTUAL LOCATION PML4 OFFSET
pub const PGMAP_VIRT_PTR: *mut u8    = 0o_177777_777_000_000_000_0000_u64 as *mut u8; //PAGE MAP VIRTUAL LOCATION POINTER
pub const PAGE_SIZE_4KIB:      usize = 0o______________________1_0000_usize;          //MEMORY PAGE SIZE (  4KiB),                            PAGE MAP LEVEL 1 ENTRY SIZE
pub const PAGE_SIZE_2MIB:      usize = 0o__________________1_000_0000_usize;          //MEMORY PAGE SIZE (  2MiB), PAGE MAP LEVEL 1 CAPACITY, PAGE MAP LEVEL 2 ENTRY SIZE
pub const PAGE_SIZE_1GIB:      usize = 0o______________1_000_000_0000_usize;          //MEMORY PAGE SIZE (  1GiB), PAGE MAP LEVEL 2 CAPACITY, PAGE MAP LEVEL 3 ENTRY SIZE
pub const PAGE_SIZE_512G:      usize = 0o__________1_000_000_000_0000_usize;          //MEMORY PAGE SIZE (512GiB), PAGE MAP LEVEL 3 CAPACITY
pub const PAGE_SIZE_256T:      usize = 0o______1_000_000_000_000_0000_usize;          //MEMORY PAGE SIZE (256TiB), PAGE MAP LEVEL 4 CAPACITY
pub const PAGE_NMBR_LVL1:      usize = 0o________________________1000_usize;          //NUMBER OF PAGE TABLE ENTRIES 1 LEVEL UP
pub const PAGE_NMBR_LVL2:      usize = 0o____________________100_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 2 LEVELS UP
pub const PAGE_NMBR_LVL3:      usize = 0o________________100_000_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 3 LEVELS UP
pub const PAGE_NMBR_LVL4:      usize = 0o____________100_000_000_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 4 LEVELS UP
