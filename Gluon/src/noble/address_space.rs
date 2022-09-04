// GLUON: Noble Address Space
// Constants related to the Noble address space layout


// HEADER
//Imports
use crate::x86_64::paging::PageMapLevel;

//Constants
//                                    SIGN PM5 PM4 PM3 PM2 PM1 OFFSET
pub const PHYSICAL_OCT:     usize = 0o_________000__________________usize; //PML4 OFFSET OF PHYSICAL MEMORY PHYSICAL LOCATION
pub const KERNEL_OCT:       usize = 0o_________400__________________usize; //PML4 OFFSET OF KERNEL VIRTUAL LOCATION
pub const STACKS_OCT:       usize = 0o_________772__________________usize; //PML4 OFFSET OF KERNEL STACKS
pub const RAMDISK_OCT:      usize = 0o_________773__________________usize; //PML4 OFFSET OF RAMDISK CREATED BY BOOTLOADER
pub const FRAME_BUFFER_OCT: usize = 0o_________774__________________usize; //PML4 OFFSET OF SCREEN BUFFERS
pub const FREE_MEMORY_OCT:  usize = 0o_________775__________________usize; //PML4 OFFSET OF FREE PHYSICAL MEMORY VIRTUAL LOCATION
pub const IDENTITY_OCT:     usize = 0o_________776__________________usize; //PML4 OFFSET OF ALL PHYSICAL MEMORY VIRTUAL LOCATION
pub const PAGE_MAP_OCT:     usize = 0o_________777__________________usize; //PML4 OFFSET OF PAGE MAP VIRTUAL LOCATION

//Limine Constants
//                                  SIGN PM5 PM4 PM3 PM2 PM1 OFFSET
pub const LIM_PYSMEM_PTR: usize = 0o_177_777_400_000_000_000_0000_usize; pub const LIM_PYSMEM_LVL: PageMapLevel = PageMapLevel::L4;
pub const LIM_KERSTK_PTR: usize = 0o_177_777_777_775_000_000_0000_usize; pub const LIM_KERSTK_LVL: PageMapLevel = PageMapLevel::L3;
pub const LIM_PYSSTK_PTR: usize = 0o_177_777_777_776_000_000_0000_usize; pub const LIM_PYSSTK_LVL: PageMapLevel = PageMapLevel::L3;
pub const LIM_KERNEL_PTR: usize = 0o_177_777_777_777_000_000_0000_usize; pub const LIM_KERNEL_LVL: PageMapLevel = PageMapLevel::L3;
