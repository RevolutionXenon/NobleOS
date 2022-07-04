// GLUON: x86-64 PAGING
// Structs, enums, traits, constants, and functions related to the contents and handling of x86-64 page tables


// HEADER
//Constants
//                                    SIGN PM5 PM4 PM3 PM2 PM1 OFFSET
pub const HIGHER_HALF_48:   usize = 0o_177_777_000_000_000_000_0000_usize; //HIGHER HALF SIGN EXTENSION IN FOUR LEVEL PAGE MAP (48-bit address space)
pub const HIGHER_HALF_57:   usize = 0o_177_000_000_000_000_000_0000_usize; //HIGHER HALF SIGN EXTENSION IN FIVE LEVEL PAGE MAP (57-bit address space)
pub const SIGN_BIT_48:      usize = 0o_000_000_400_000_000_000_0000_usize; //SIGN BIT IN FOUR LEVEL PAGE MAP (48-bit address space)
pub const SIGN_BIT_57:      usize = 0o_000_400_000_000_000_000_0000_usize; //SIGN BIT IN FIVE LEVEL PAGE MAP (57-bit address space)
pub const PAGE_MASK_OFFS:   usize = 0o_000_000_000_000_000_000_7777_usize; //ADDRESS MASK OF OFFSET
pub const PAGE_MASK_PML1:   usize = 0o_000_000_000_000_000_777_0000_usize; //ADDRESS MASK OF INDEX INTO PAGE MAP LEVEL 1
pub const PAGE_MASK_PML2:   usize = 0o_000_000_000_000_777_000_0000_usize; //ADDRESS MASK OF INDEX INTO PAGE MAP LEVEL 2
pub const PAGE_MASK_PML3:   usize = 0o_000_000_000_777_000_000_0000_usize; //ADDRESS MASK OF INDEX INTO PAGE MAP LEVEL 3
pub const PAGE_MASK_PML4:   usize = 0o_000_000_777_000_000_000_0000_usize; //ADDRESS MASK OF INDEX INTO PAGE MAP LEVEL 4
pub const PAGE_MASK_PML5:   usize = 0o_000_777_000_000_000_000_0000_usize; //ADDRESS MASK OF INDEX INTO PAGE MAP LEVEL 5
pub const PAGE_SIZE_4KIB:   usize = 0o_000_000_000_000_000_001_0000_usize; //MEMORY PAGE SIZE (  4KiB), PAGE MAP LEVEL 1 ENTRY SIZE
pub const PAGE_SIZE_2MIB:   usize = 0o_000_000_000_000_001_000_0000_usize; //MEMORY PAGE SIZE (  2MiB), PAGE MAP LEVEL 2 ENTRY SIZE, PAGE MAP LEVEL 1 CAPACITY
pub const PAGE_SIZE_1GIB:   usize = 0o_000_000_000_001_000_000_0000_usize; //MEMORY PAGE SIZE (  1GiB), PAGE MAP LEVEL 3 ENTRY SIZE, PAGE MAP LEVEL 2 CAPACITY
pub const PAGE_SIZE_512G:   usize = 0o_000_000_001_000_000_000_0000_usize; //MEMORY PAGE SIZE (512GiB),                              PAGE MAP LEVEL 3 CAPACITY
pub const PAGE_SIZE_256T:   usize = 0o_000_001_000_000_000_000_0000_usize; //MEMORY PAGE SIZE (256TiB),                              PAGE MAP LEVEL 4 CAPACITY
pub const PAGE_SIZE_128P:   usize = 0o_001_000_000_000_000_000_0000_usize; //MEMORY PAGE SIZE (128PiB),                              PAGE MAP LEVEL 5 CAPACITY
pub const PAGE_NUMBER_1:    usize = 0o_000_000_000_000_000_000_1000_usize; //NUMBER OF PAGE TABLE ENTRIES 1 LEVELS UP (               512)
pub const PAGE_NUMBER_2:    usize = 0o_000_000_000_000_000_100_0000_usize; //NUMBER OF PAGE TABLE ENTRIES 2 LEVELS UP (           262,144)
pub const PAGE_NUMBER_3:    usize = 0o_000_000_000_000_100_000_0000_usize; //NUMBER OF PAGE TABLE ENTRIES 3 LEVELS UP (       134,217,728)
pub const PAGE_NUMBER_4:    usize = 0o_000_000_000_100_000_000_0000_usize; //NUMBER OF PAGE TABLE ENTRIES 4 LEVELS UP (    68,719,476,736)
pub const PAGE_NUMBER_5:    usize = 0o_000_000_100_000_000_000_0000_usize; //NUMBER OF PAGE TABLE ENTRIES 5 LEVELS UP (35,184,372,088,832)
pub const KIB:              usize = 0o_000_000_000_000_000_000_2000_usize; //ONE KIBIBYTE
pub const MIB:              usize = 0o_000_000_000_000_000_400_0000_usize; //ONE MEBIBYTE
pub const GIB:              usize = 0o_000_000_000_001_000_000_0000_usize; //ONE GIBIBYTE
pub const TIB:              usize = 0o_000_000_002_000_000_000_0000_usize; //ONE TEBIBYTE
pub const PIB:              usize = 0o_000_004_000_000_000_000_0000_usize; //ONE PEBIBYTE

//Imports
use crate::noble::return_code::ReturnCode;


// ADDRESSES
//Conversion from 9-bit specifiers to addresses (4-level paging)
pub fn oct_to_usize_4(pml4: usize, pml3: usize, pml2: usize, pml1: usize, offset: usize)   -> Result<usize,   &'static str> {
    if pml4   >= 512  {return Err("O4 to Pointer: PML4 oct out of bounds.")}
    if pml3   >= 512  {return Err("O4 to Pointer: PML3 oct out of bounds.")}
    if pml2   >= 512  {return Err("O4 to Pointer: PML2 oct out of bounds.")}
    if pml1   >= 512  {return Err("O4 to Pointer: PML1 oct out of bounds.")}
    if offset >= 4096 {return Err("O4 to Pointer: Offset out of bounds.")}
    let mut result: usize = if pml4 >= 0o400 {HIGHER_HALF_48} else {0};
    result |= pml4 << (0o14 + 0o11 + 0o11 + 0o11);
    result |= pml3 << (0o14 + 0o11 + 0o11);
    result |= pml2 << (0o14 + 0o11);
    result |= pml1 << (0o14);
    result |= offset;
    Ok(result)
}
pub fn oct_to_pointer_4(pml4: usize, pml3: usize, pml2: usize, pml1: usize, offset: usize) -> Result<*mut u8, &'static str> {
    Ok(oct_to_usize_4(pml4, pml3, pml2, pml1, offset)? as *mut u8)
}
pub fn oct4_to_usize(pml4: usize)                                                          -> Result<usize,   &'static str> {
    oct_to_usize_4(pml4, 0, 0, 0, 0)
}
pub fn oct4_to_pointer(pml4: usize)                                                        -> Result<*mut u8, &'static str> {
    oct_to_pointer_4(pml4, 0, 0, 0, 0)
}

//Conversion from 9-bit specifiers to addresses (5-level paging)
pub fn oct_to_usize_5(pml5: usize, pml4: usize, pml3: usize, pml2: usize, pml1: usize, offset: usize)   -> Result<usize,   &'static str> {
    if pml5   >= PAGE_NUMBER_1  {return Err("O5 to Pointer: PML5 oct out of bounds.")}
    if pml4   >= PAGE_NUMBER_1  {return Err("O5 to Pointer: PML4 oct out of bounds.")}
    if pml3   >= PAGE_NUMBER_1  {return Err("O5 to Pointer: PML3 oct out of bounds.")}
    if pml2   >= PAGE_NUMBER_1  {return Err("O5 to Pointer: PML2 oct out of bounds.")}
    if pml1   >= PAGE_NUMBER_1  {return Err("O5 to Pointer: PML1 oct out of bounds.")}
    if offset >= PAGE_SIZE_4KIB {return Err("O5 to Pointer: Offset out of bounds.")}
    let mut result: usize = if pml4 >= 0o400 {HIGHER_HALF_48} else {0};
    result |= pml4 << (0o14 + 0o11 + 0o11 + 0o11);
    result |= pml3 << (0o14 + 0o11 + 0o11);
    result |= pml2 << (0o14 + 0o11);
    result |= pml1 << (0o14);
    result |= offset;
    Ok(result)
}
pub fn oct_to_pointer_5(pml5: usize, pml4: usize, pml3: usize, pml2: usize, pml1: usize, offset: usize) -> Result<*mut u8, &'static str> {
    Ok(oct_to_usize_5(pml5, pml4, pml3, pml2, pml1, offset)? as *mut u8)
}
pub fn oct5_to_usize(pml5: usize)                                                                       -> Result<usize,   &'static str> {
    oct_to_usize_5(pml5, 0, 0, 0, 0, 0)
}
pub fn oct5_to_pointer(pml5: usize)                                                                     -> Result<*mut u8, &'static str> {
    oct_to_pointer_5(pml5, 0, 0, 0, 0, 0)
}

//Check an address is canonical
pub fn canonical_48(address: LinearAddress) -> Result<(), ReturnCode> {
    let mask: usize = SIGN_BIT_48 | HIGHER_HALF_48;
    let masked: usize = address.0 & mask;
    match masked == mask || masked == 0 {
        true => Ok(()),
        false => Err(ReturnCode::NonCanonicalAddress),
    }
}

//Extract an index
pub fn extract_index(address: LinearAddress, level: PageMapLevel) -> usize {
    match level {
        PageMapLevel::L5 => (address.0 & PAGE_MASK_PML5) >> 48,
        PageMapLevel::L4 => (address.0 & PAGE_MASK_PML4) >> 39,
        PageMapLevel::L3 => (address.0 & PAGE_MASK_PML3) >> 30,
        PageMapLevel::L2 => (address.0 & PAGE_MASK_PML2) >> 21,
        PageMapLevel::L1 => (address.0 & PAGE_MASK_PML1) >> 12,
    }
}

//Get entry capacity
pub fn page_size(level: PageMapLevel) -> usize {
    match level {
        PageMapLevel::L5 => PAGE_SIZE_256T,
        PageMapLevel::L4 => PAGE_SIZE_512G,
        PageMapLevel::L3 => PAGE_SIZE_1GIB,
        PageMapLevel::L2 => PAGE_SIZE_2MIB,
        PageMapLevel::L1 => PAGE_SIZE_4KIB,
    }
}


// PAGING
//Page Allocator
pub trait PageAllocator {
    fn allocate_page     (&self)                            -> Result<PhysicalAddress, &'static str>;
    fn deallocate_page   (&self, physical: PhysicalAddress) -> Result<(),              &'static str>;
    fn physical_to_linear(&self, physical: PhysicalAddress) -> Result<LinearAddress,   &'static str>;
}

//Physical Address
#[repr(transparent)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct PhysicalAddress(pub usize);
impl PhysicalAddress {
    pub fn add(&self, offset: usize) -> Self {
        Self(self.0 + offset)
    }
}

//Linear Address
#[repr(transparent)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct LinearAddress(pub usize);
impl LinearAddress {
    pub fn add(&self, offset: usize) -> Self {
        Self(self.0 + offset)
    }
    pub fn sub(&self, offset: usize) -> Self {
        Self(self.0 - offset)
    }
}

//Page Table
pub struct PageMap<'s>{
    pub linear:     LinearAddress,
    pub physical:   PhysicalAddress,
    pub map_level:      PageMapLevel,
    page_allocator: &'s dyn PageAllocator,
}
impl<'s> PageMap<'s> {
    //Constructor
    pub fn new(physical: PhysicalAddress, map_level: PageMapLevel, page_allocator: &'s dyn PageAllocator) -> Result<Self, &'static str> {
        if physical.0 % PAGE_SIZE_4KIB != 0 {return Err("Page Map: Location not aligned to 4KiB boundary.")}
        Ok(Self {
            linear: page_allocator.physical_to_linear(physical)?,
            physical,
            map_level,
            page_allocator,
        })
    }

    //Get an entry from a location
    pub fn read_entry(&self, position: usize) -> Result<PageMapEntry, &'static str> {
        if position >= PAGE_NUMBER_1 {return Err("Page Map: Entry index out of bounds during read.")}
        let data = unsafe{*((self.linear.0 as *mut u64).add(position))};
        PageMapEntry::from_u64(data, self.map_level)
    }

    //Write an entry to a location
    pub fn write_entry(&self, position: usize, entry: PageMapEntry) -> Result<(), &'static str> {
        if position >= PAGE_NUMBER_1 {return Err("Page Map: Entry index out of bounds during write.")}
        if entry.entry_level != self.map_level {return Err("Page Map: Entry level does not match map level.")}
        let data = entry.to_u64()?;
        unsafe {*((self.linear.0 as *mut u64).add(position)) = data}
        Ok(())
    }

    //Map pages from a physical offset and within-map offset
    pub fn  map_pages_offset_4kib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        match self.map_level {
            PageMapLevel::L1 => {self.map_pages_offset_pml1_4kib(physical_offset, map_offset, size, write, user, execute_disable)}
            PageMapLevel::L2 => {self.map_pages_offset_pml2_4kib(physical_offset, map_offset, size, write, user, execute_disable)}
            PageMapLevel::L3 => {self.map_pages_offset_pml3_4kib(physical_offset, map_offset, size, write, user, execute_disable)}
            _ => Err("Page Map: Map pages offset 4KiB not implemented for this map level.")
        }
    }
    fn map_pages_offset_pml1_4kib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        //Check Parameters
        if self.map_level                       != PageMapLevel::L1 {return Err("Page Map: Offset PML1 4KiB called on page map of wrong level.")}
        if physical_offset.0   % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Offset PML1 4KiB called on unaligned physical address.")}
        if map_offset as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Offset PML1 4KiB called on unaligned map offset.")}
        if map_offset +  size  > PAGE_SIZE_2MIB                     {return Err("Page Map: Offset PML1 4KiB called on offset and size that does not fit within map boundaries.")}
        //Position variables
        let pages:    usize = size       / PAGE_SIZE_4KIB + if size%PAGE_SIZE_4KIB != 0 {1} else {0};
        let position: usize = map_offset / PAGE_SIZE_4KIB;
        //Loop
        for i in 0..pages {
            self.write_entry(i+position, PageMapEntry::new(PageMapLevel::L1, PageMapEntryType::Memory, physical_offset.add(i*PAGE_SIZE_4KIB), write, user, execute_disable)?)?;
        }
        //Return
        Ok(())
    }
    fn map_pages_offset_pml2_4kib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        //Check Parameters
        if self.map_level                       != PageMapLevel::L2 {return Err("Page Map: Offset PML2 4KiB called on page map of wrong level.")}
        if physical_offset.0   % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Offset PML2 4KiB called on unaligned physical address.")}
        if map_offset as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Offset PML2 4KiB called on unaligned map offset.")}
        if map_offset +  size  > PAGE_SIZE_1GIB                     {return Err("Page Map: Offset PML2 4KiB called on offset and size that does not fit within map boundaries.")}
        //Position Variables
        let start_position: usize =  map_offset         / PAGE_SIZE_2MIB;
        let start_size:     usize =  map_offset         % PAGE_SIZE_2MIB;
        let align_size:     usize = (map_offset + size) % PAGE_SIZE_2MIB;
        let end_position:   usize = (map_offset + size) / PAGE_SIZE_2MIB + if align_size != 0 {1} else {0};
        let end_size:       usize = if align_size == 0   {PAGE_SIZE_2MIB} else {align_size};
        //Loop
        for position in start_position..end_position {
            //Retrieve PML1
            let entry = match self.read_entry(position) {
                Ok(entry) => {
                    if entry.present {
                        entry
                    }
                    else {
                        let new_entry = PageMapEntry::new(PageMapLevel::L2, PageMapEntryType::Table, self.page_allocator.allocate_page()?, write, user, execute_disable)?;
                        self.write_entry(position, new_entry)?;
                        new_entry
                    }
                },
                Err(_) => {
                    let new_entry = PageMapEntry::new(PageMapLevel::L2, PageMapEntryType::Table, self.page_allocator.allocate_page()?, write, user, execute_disable)?;
                    self.write_entry(position, new_entry)?;
                    new_entry
                },
            };
            let pml1 = PageMap::new(entry.physical, PageMapLevel::L1, self.page_allocator)?;
            //Map within PML1
            if position == start_position && position == end_position-1 {
                pml1.map_pages_offset_pml1_4kib(physical_offset, start_size, size, write, user, execute_disable)?;
            }
            else if position == start_position {
                pml1.map_pages_offset_pml1_4kib(physical_offset, start_size, PAGE_SIZE_2MIB-start_size, write, user, execute_disable)?;
            }
            else if position == end_position-1 {
                pml1.map_pages_offset_pml1_4kib(physical_offset.add((position-start_position)*PAGE_SIZE_2MIB - start_size), 0, end_size, write, user, execute_disable)?;
            }
            else {
                pml1.map_pages_offset_pml1_4kib(physical_offset.add((position-start_position)*PAGE_SIZE_2MIB - start_size), 0, PAGE_SIZE_2MIB, write, user, execute_disable)?;
            }
        }
        //Return
        Ok(())
    }
    fn map_pages_offset_pml3_4kib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        //Check Parameters
        if self.map_level                       != PageMapLevel::L3 {return Err("Page Map: Offset PML3 4KiB called on page map of wrong level.")}
        if physical_offset.0   % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Offset PML3 4KiB called on unaligned physical address.")}
        if map_offset as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Offset PML3 4KiB called on unaligned map offset.")}
        if map_offset +  size  > PAGE_SIZE_512G                     {return Err("Page Map: Offset PML3 4KiB called on offset and size that does not fit within map boundaries.")}
        //Position Variables
        let start_position: usize =  map_offset         / PAGE_SIZE_1GIB;
        let start_size:     usize =  map_offset         % PAGE_SIZE_1GIB;
        let align_size:     usize = (map_offset + size) % PAGE_SIZE_1GIB;
        let end_position:   usize = (map_offset + size) / PAGE_SIZE_1GIB + if align_size != 0 {1} else {0};
        let end_size:       usize = if align_size == 0   {PAGE_SIZE_1GIB} else {align_size};
        //Loop
        for position in start_position..end_position {
            //Retrieve PML2
            let entry = match self.read_entry(position) {
                Ok(entry) => {
                    if entry.present {
                        entry
                    }
                    else {
                        let new_entry = PageMapEntry::new(PageMapLevel::L3, PageMapEntryType::Table, self.page_allocator.allocate_page()?, write, user, execute_disable)?;
                        self.write_entry(position, new_entry)?;
                        new_entry
                    }
                },
                Err(_) => {
                    let new_entry = PageMapEntry::new(PageMapLevel::L3, PageMapEntryType::Table, self.page_allocator.allocate_page()?, write, user, execute_disable)?;
                    self.write_entry(position, new_entry)?;
                    new_entry
                },
            };
            let pml2 = PageMap::new(entry.physical, PageMapLevel::L2, self.page_allocator)?;
            //Map within PML2
            if position == start_position && position == end_position-1 {
                pml2.map_pages_offset_pml2_4kib(physical_offset, start_size, size, write, user, execute_disable)?;
            }
            else if position == start_position {
                pml2.map_pages_offset_pml2_4kib(physical_offset, start_size, PAGE_SIZE_1GIB - start_size, write, user, execute_disable)?;
            }
            else if position == end_position-1 {
                pml2.map_pages_offset_pml2_4kib(physical_offset.add((position-start_position)*PAGE_SIZE_1GIB - start_size), 0, end_size, write, user, execute_disable)?;
            }
            else {
                pml2.map_pages_offset_pml2_4kib(physical_offset.add((position-start_position)*PAGE_SIZE_1GIB - start_size), 0, PAGE_SIZE_1GIB, write, user, execute_disable)?;
            }
        }
        //Return
        Ok(())
    }
    
    pub fn  map_pages_offset_2mib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        match self.map_level {
            PageMapLevel::L2 => {self.map_pages_offset_pml2_2mib(physical_offset, map_offset, size, write, user, execute_disable)}
            PageMapLevel::L3 => {self.map_pages_offset_pml3_2mib(physical_offset, map_offset, size, write, user, execute_disable)}
            _ => Err("Page Map: Map pages offset 2MiB not implemented for this map level.")
        }
    }
    fn map_pages_offset_pml2_2mib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        if self.map_level                       != PageMapLevel::L2 {return Err("Page Map: Offset PML2 2MiB called on page map of wrong level.")}
        if physical_offset.0   % PAGE_SIZE_2MIB != 0                {return Err("Page Map: Offset PML2 2MiB called on unaligned physical address.")}
        if map_offset as usize % PAGE_SIZE_2MIB != 0                {return Err("Page Map: Offset PML2 2MiB called on unaligned map offset.")}
        if map_offset +  size  > PAGE_SIZE_1GIB                     {return Err("Page Map: Offset PML2 2MiB called on offset and size that does not fit within map boundaries.")}
        //Position variables
        let pages:    usize = size       / PAGE_SIZE_2MIB + if size%PAGE_SIZE_2MIB != 0 {1} else {0};
        let position: usize = map_offset / PAGE_SIZE_2MIB;
        //Loop
        for i in 0..pages {
            self.write_entry(i+position, PageMapEntry::new(PageMapLevel::L2, PageMapEntryType::Memory, physical_offset.add(i*PAGE_SIZE_2MIB), write, user, execute_disable)?)?;
        }
        //Return
        Ok(())
    }
    fn map_pages_offset_pml3_2mib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        if self.map_level                       != PageMapLevel::L3 {return Err("Page Map: Offset PML3 2MiB called on page map of wrong level.")}
        if physical_offset.0   % PAGE_SIZE_2MIB != 0                {return Err("Page Map: Offset PML3 2MiB called on unaligned physical address.")}
        if map_offset as usize % PAGE_SIZE_2MIB != 0                {return Err("Page Map: Offset PML3 2MiB called on unaligned map offset.")}
        if map_offset +  size  > PAGE_SIZE_512G                     {return Err("Page Map: Offset PML3 2MiB called on offset and size that does not fit within map boundaries.")}
        //Position Variables
        let start_position: usize =  map_offset         / PAGE_SIZE_1GIB;
        let start_size:     usize =  map_offset         % PAGE_SIZE_1GIB;
        let align_size:     usize = (map_offset + size) % PAGE_SIZE_1GIB;
        let end_position:   usize = (map_offset + size) / PAGE_SIZE_1GIB + if align_size != 0 {1} else {0};
        let end_size:       usize = if align_size == 0   {PAGE_SIZE_1GIB} else {align_size};
        //Loop
        for position in start_position..end_position {
            //Retrieve PML2
            let entry = match self.read_entry(position) {
                Ok(entry) => {
                    if entry.present {
                        entry
                    }
                    else {
                        let new_entry = PageMapEntry::new(PageMapLevel::L3, PageMapEntryType::Table, self.page_allocator.allocate_page()?, write, user, execute_disable)?;
                        self.write_entry(position, new_entry)?;
                        new_entry
                    }
                },
                Err(_) => {
                    let new_entry = PageMapEntry::new(PageMapLevel::L3, PageMapEntryType::Table, self.page_allocator.allocate_page()?, write, user, execute_disable)?;
                    self.write_entry(position, new_entry)?;
                    new_entry
                },
            };
            let pml2 = PageMap::new(entry.physical, PageMapLevel::L2, self.page_allocator)?;
            //Map within PML2
            if position == start_position && position == end_position-1 {
                pml2.map_pages_offset_pml2_2mib(physical_offset, start_size, size, write, user, execute_disable)?;
            }
            else if position == start_position {
                pml2.map_pages_offset_pml2_2mib(physical_offset, start_size, PAGE_SIZE_1GIB - start_size, write, user, execute_disable)?;
            }
            else if position == end_position-1 {
                pml2.map_pages_offset_pml2_2mib(physical_offset.add((position-start_position)*PAGE_SIZE_1GIB - start_size), 0, end_size, write, user, execute_disable)?;
            }
            else {
                pml2.map_pages_offset_pml2_2mib(physical_offset.add((position-start_position)*PAGE_SIZE_1GIB - start_size), 0, PAGE_SIZE_1GIB, write, user, execute_disable)?;
            }
        }
        //Return
        Err("Unfinished")
    }

    pub fn  map_pages_offset_1gib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        match self.map_level {
            PageMapLevel::L3 => {self.map_pages_offset_pml3_1gib(physical_offset, map_offset, size, write, user, execute_disable)}
            _ => Err("Page Map: Map pages offset 2MiB not implemented for this map level.")
        }
    }
    fn map_pages_offset_pml3_1gib(&self, physical_offset: PhysicalAddress, map_offset: usize, size: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        if self.map_level                       != PageMapLevel::L3 {return Err("Page Map: Offset PML3 1GiB called on page map of wrong level.")}
        if physical_offset.0   % PAGE_SIZE_1GIB != 0                {return Err("Page Map: Offset PML3 1GiB called on unaligned physical address.")}
        if map_offset as usize % PAGE_SIZE_1GIB != 0                {return Err("Page Map: Offset PML3 1GiB called on unaligned map offset.")}
        if map_offset +  size  > PAGE_SIZE_512G                     {return Err("Page Map: Offset PML3 1GiB called on offset and size that does not fit within map boundaries.")}
        //Position variables
        let pages:    usize = size       / PAGE_SIZE_1GIB + if size%PAGE_SIZE_1GIB != 0 {1} else {0};
        let position: usize = map_offset / PAGE_SIZE_1GIB;
        //Loop
        for i in 0..pages {
            self.write_entry(i+position, PageMapEntry::new(PageMapLevel::L3, PageMapEntryType::Memory, physical_offset.add(i*PAGE_SIZE_1GIB), write, user, execute_disable)?)?;
        }
        //Return
        Ok(())
    }

    //Map pages from a list of physical pages and within-map offset
    pub fn  map_pages_group_4kib(&self, group: &[PhysicalAddress], page_offset: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        match self.map_level {
            PageMapLevel::L1 => {self.map_pages_group_pml1_4kib(group, page_offset, write, user, execute_disable)}
            PageMapLevel::L2 => {self.map_pages_group_pml2_4kib(group, page_offset, write, user, execute_disable)}
            PageMapLevel::L3 => {self.map_pages_group_pml3_4kib(group, page_offset, write, user, execute_disable)}
            _ => Err("Page Map: Map pages group 4KiB not implemented for this map level.")
        }
    }
    fn map_pages_group_pml1_4kib(&self, group: &[PhysicalAddress], page_offset: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        //Parameters
        if  self.map_level                   != PageMapLevel::L1 {return Err("Page Map: Group PML1 4KiB called on page map of wrong level.")}
        if  page_offset + group.len()        >  PAGE_NUMBER_1    {return Err("Page Map: Group PML1 4KiB called on offset and group size that does not fit within map boundaries.")}
        //Loop
        for (i, page) in group.iter().enumerate() {
            let address = page.0;
            if address % PAGE_SIZE_4KIB != 0 {return Err("Page Map: Group PML1 4KiB called with unaligned addresses in group.")}
            self.write_entry(page_offset + i, PageMapEntry::new(PageMapLevel::L1, PageMapEntryType::Memory, group[i], write, user, execute_disable)?)?;
        }
        //Return
        Ok(())
    }
    fn map_pages_group_pml2_4kib(&self, group: &[PhysicalAddress], page_offset: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        //Check Parameters
        if  self.map_level            != PageMapLevel::L2 {return Err("Page Map: Group PML2 4KiB called on page map of wrong level.")}
        if  page_offset + group.len() >  PAGE_NUMBER_2    {return Err("Page Map: Group PML2 4KiB called on offset and group size that does not fit within map boundaries.")}
        //Position variables
        let first_position:  usize = page_offset / PAGE_NUMBER_1;
        let first_offset:    usize = page_offset % PAGE_NUMBER_1;
        let last_position:   usize = (page_offset + group.len() - 1) / PAGE_NUMBER_1;
        let mut group_index: usize = 0;
        //Loop
        for position in first_position..last_position+1 {
            let pml2e = self.read_entry(position)?;
            let pml1 = match (pml2e.present, pml2e.entry_type) {
                (false, _)                        => {
                    let ph_new = self.page_allocator.allocate_page()?;
                    self.write_entry(position, PageMapEntry::new(PageMapLevel::L2, PageMapEntryType::Table, ph_new, write, user, execute_disable)?)?;
                    PageMap::new(ph_new, PageMapLevel::L1, self.page_allocator)?
                },
                (true,  PageMapEntryType::Memory) => return Err("Page Map: Group PML2 4KiB called to write over page map which contains 2MiB entries."),
                (true,  PageMapEntryType::Table)  => PageMap::new(pml2e.physical, PageMapLevel::L1, self.page_allocator)?,
            };
            if position == first_position && position == last_position {
                pml1.map_pages_group_pml1_4kib(group, first_offset, write, user, execute_disable)?;
            }
            else if position == first_position {
                group_index += PAGE_NUMBER_1 - first_offset;
                pml1.map_pages_group_pml1_4kib(&group[0..group_index], first_offset, write, user, execute_disable)?;
            }
            else if position == last_position {
                pml1.map_pages_group_pml1_4kib(&group[group_index..], 0, write, user, execute_disable)?;
            }
            else {
                pml1.map_pages_group_pml1_4kib(&group[group_index..group_index+PAGE_NUMBER_1], 0, write, user, execute_disable)?;
                group_index += PAGE_NUMBER_1;
            }
        }
        //Return
        Ok(())
    }
    fn map_pages_group_pml3_4kib(&self, group: &[PhysicalAddress], page_offset: usize, write: bool, user: bool, execute_disable: bool) -> Result<(), &'static str> {
        //Check Parameters
        if  self.map_level                  != PageMapLevel::L3 {return Err("Page Map: Group PML3 4KiB called on page map of wrong level.")}
        if  page_offset + group.len()       >  PAGE_NUMBER_3    {return Err("Page Map: Group PML2 4KiB called on offset and group size that does not fit within map boundaries.")}
        //Position variables
        let first_position:  usize = page_offset / PAGE_NUMBER_2;
        let first_offset:    usize = page_offset % PAGE_NUMBER_2;
        let last_position:   usize = (page_offset + group.len() - 1) / PAGE_NUMBER_2;
        let mut group_index: usize = 0;
        //Loop
        for position in first_position..last_position+1 {
            let pml3e = self.read_entry(position)?;
            let pml2 = match (pml3e.present, pml3e.entry_type) {
                (false, _)                        => {
                    let ph_new = self.page_allocator.allocate_page()?;
                    self.write_entry(position, PageMapEntry::new(PageMapLevel::L3, PageMapEntryType::Table, ph_new, write, user, execute_disable)?)?;
                    PageMap::new(ph_new, PageMapLevel::L2, self.page_allocator)?
                },
                (true,  PageMapEntryType::Memory) => return Err("Page Map: Group PML3 4KiB called to write over page map which contains 1GiB entries."),
                (true,  PageMapEntryType::Table)  => PageMap::new(pml3e.physical, PageMapLevel::L2, self.page_allocator)?,
            };
            if position == first_position && position == last_position {
                pml2.map_pages_group_pml2_4kib(group, first_offset, write, user, execute_disable)?;
            }
            else if position == first_position {
                group_index += PAGE_NUMBER_2 - first_offset;
                pml2.map_pages_group_pml2_4kib(&group[0..group_index], first_offset, write, user, execute_disable)?;
            }
            else if position == last_position {
                pml2.map_pages_group_pml2_4kib(&group[group_index..], 0, write, user, execute_disable)?;
            }
            else {
                pml2.map_pages_group_pml2_4kib(&group[group_index..group_index+PAGE_NUMBER_2], 0, write, user, execute_disable)?;
                group_index += PAGE_NUMBER_2;
            }
        }
        //Return
        Ok(())
    }
}

//Page Table Entry
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct PageMapEntry {
    pub entry_level:     PageMapLevel,
    pub entry_type:      PageMapEntryType, //Bit 7 in some cirumstances, indicates page refers to memory when it could refer to a table
    pub physical:        PhysicalAddress,
    pub present:         bool, //ALL: Bit 0, indicates entry exists
    pub write:           bool, //ALL: Bit 1, indicates page may be written to
    pub user:            bool, //ALL: Bit 2, indicates page can only be accessed in Ring 0
    pub write_through:   bool, //ALL: Bit 3, something about how memory access works
    pub cache_disable:   bool, //ALL: Bit 4, something else about how memory access works
    pub accessed:        bool, //ALL: Bit 5, indicates that a page has been accessed
    pub dirty:           Option<bool>, //MEMORY: Bit 6, indicates page has been written to
    pub attribute_table: Option<bool>, //MEMORY: Bit 7 (L1) or Bit 12 (L2, L3), indicates yet another thing about how memory access works
    pub global:          Option<bool>, //MEMORY: Bit 8,
    pub in_use:          bool, //ALL: Bit 52, indicates to the operating system that a page map entry is valid regardless of the state of the present bit
    pub execute_disable: bool, //ALL: Bit 63, indicates code may not be executed from this page
}
impl PageMapEntry {
    //Read from u64, intended to read a page table entry from RAM
    pub fn from_u64(data: u64, entry_level: PageMapLevel) -> Result<Self, &'static str> {
        let entry_type = {
            if      entry_level == PageMapLevel::L5 || entry_level == PageMapLevel::L4 {PageMapEntryType::Table}
            else if entry_level == PageMapLevel::L3 || entry_level == PageMapLevel::L2 { 
                if data & (1<<7) > 0                                                   {PageMapEntryType::Memory}
                else                                                                   {PageMapEntryType::Table}}
            else                                                                       {PageMapEntryType::Memory}
        };
        Ok(Self {
            entry_level,
            entry_type,
            physical: PhysicalAddress(match (entry_level, entry_type) {
                (PageMapLevel::L5, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L4, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L3, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L2, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L3, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_000_000_0000_u64,
                (PageMapLevel::L2, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_777_000_0000_u64,
                (PageMapLevel::L1, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_777_777_0000_u64,
                _ => {return Err("Page Table Entry: Invalid combination of level and entry type found.")}
            } as usize),
            present:                                                 data & (1<<0o00) > 0,
            write:                                                   data & (1<<0o01) > 0,
            user:                                              data & (1<<0o02) > 0,
            write_through:                                           data & (1<<0o03) > 0,
            cache_disable:                                           data & (1<<0o04) > 0,
            accessed:                                                data & (1<<0o05) > 0,
            dirty: match entry_type {
                                PageMapEntryType::Memory     => Some(data & (1<<0o06) > 0),
                                PageMapEntryType::Table      => None,
            },
            attribute_table: match (entry_level, entry_type) {
                (PageMapLevel::L3, PageMapEntryType::Memory) => Some(data & (1<<0o14) > 0),
                (PageMapLevel::L2, PageMapEntryType::Memory) => Some(data & (1<<0o14) > 0),
                (PageMapLevel::L1, PageMapEntryType::Memory) => Some(data & (1<<0o07) > 0),
                _                                            => None,
            },
            global: match entry_type {
                                PageMapEntryType::Memory     => Some(data & (1<<0o10) > 0),
                                PageMapEntryType::Table      => None,
            },
            in_use:                                                  data & (1<<0o64) > 0,
            execute_disable:                                         data & (1<<0o77) > 0,
        })
    }
    
    //Convert to u64, intended to aid in writing a page table entry into RAM
    pub fn to_u64(&self) -> Result<u64, &'static str> {
        let mut result: u64 = 0;
        result |= match (self.entry_level, self.entry_type) {
            (PageMapLevel::L5, PageMapEntryType::Table)  => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L4, PageMapEntryType::Table)  => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L3, PageMapEntryType::Table)  => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L2, PageMapEntryType::Table)  => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L3, PageMapEntryType::Memory) => self.physical.0 as u64 & 0o_000_777_777_777_000_000_0000_u64,
            (PageMapLevel::L2, PageMapEntryType::Memory) => self.physical.0 as u64 & 0o_000_777_777_777_777_000_0000_u64,
            (PageMapLevel::L1, PageMapEntryType::Memory) => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            _ => {return Err("Page Table Entry: Invalid combination of level and entry type in struct.")}
        };
        if self.present       {result |= 1<<0o00}
        if self.write         {result |= 1<<0o01}
        if self.user    {result |= 1<<0o02}
        if self.write_through {result |= 1<<0o03}
        if self.cache_disable {result |= 1<<0o04}
        if self.accessed      {result |= 1<<0o05}
        if self.entry_type == PageMapEntryType::Memory {
            if self.dirty.is_some() && self.dirty.unwrap() {result |= 1<<0o06}
            if self.entry_level == PageMapLevel::L3 || self.entry_level == PageMapLevel::L2 {
                result |= 1<<0o07;
                if self.attribute_table.is_some() && self.attribute_table.unwrap() {result |= 1<<0o14}
            }
            else if self.entry_level == PageMapLevel::L1 && self.attribute_table.is_some() && self.attribute_table.unwrap() {result |= 1<<0o07}
        }
        if self.execute_disable {result |= 1<<0o77}
        Ok(result)
    }

    //New
    pub fn new(entry_level: PageMapLevel, entry_type: PageMapEntryType, address: PhysicalAddress, write: bool, user: bool, execute_disable: bool) -> Result<Self, &'static str> {
        match (entry_level, entry_type) {
            (PageMapLevel::L5, PageMapEntryType::Table)  => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            (PageMapLevel::L4, PageMapEntryType::Table)  => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            (PageMapLevel::L3, PageMapEntryType::Table)  => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            (PageMapLevel::L2, PageMapEntryType::Table)  => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            (PageMapLevel::L3, PageMapEntryType::Memory) => {if address.0 as usize % PAGE_SIZE_1GIB != 0 {return Err("Page Table Entry: Address is not aligned to a 1GiB boundary.")}},
            (PageMapLevel::L2, PageMapEntryType::Memory) => {if address.0 as usize % PAGE_SIZE_2MIB != 0 {return Err("Page Table Entry: Address is not aligned to a 2MiB boundary.")}},
            (PageMapLevel::L1, PageMapEntryType::Memory) => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            _ => {return Err("Page Table Entry: Invalid combination of level and entry type in constructor.")}
        };
        Ok(Self {
            entry_level,
            entry_type,
            physical: address,
            present:         true,
            write,
            user,
            write_through:   false,
            cache_disable:   false,
            accessed:        false,
            dirty:           if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            attribute_table: if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            global:          if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            in_use:          true,
            execute_disable,
        })
    }
}

//Page Map Level
#[derive(PartialEq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum PageMapLevel {
    L5 = 5,
    L4 = 4,
    L3 = 3,
    L2 = 2,
    L1 = 1,
}
impl PageMapLevel {
    pub fn sub(self) -> Result<Self, ReturnCode> {
        match self {
            PageMapLevel::L5 => Ok(Self::L4),
            PageMapLevel::L4 => Ok(Self::L3),
            PageMapLevel::L3 => Ok(Self::L2),
            PageMapLevel::L2 => Ok(Self::L1),
            PageMapLevel::L1 => Err(ReturnCode::InvalidData),
        }
    }
}


//Page Map Entry Type
#[derive(PartialEq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum PageMapEntryType {
    Memory,
    Table,
}

// NEW SYSTEM
pub struct PageMap2 {
    pub linear:     LinearAddress,
    pub map_level:  PageMapLevel,
}
impl PageMap2 {
    //Constructor
    pub fn new(linear: LinearAddress, map_level: PageMapLevel) -> Result<Self, ReturnCode> {
        if linear.0 % PAGE_SIZE_4KIB != 0 {return Err(ReturnCode::UnalignedAddress)}
        Ok(Self {
            linear,
            map_level,
        })
    }

    //Get an entry from a location
    pub fn read_entry(&self, position: usize) -> Result<PageMapEntry2, ReturnCode> {
        if position >= PAGE_NUMBER_1 {return Err(ReturnCode::IndexOutOfBounds)}
        let data = unsafe{*((self.linear.0 as *mut u64).add(position))};
        PageMapEntry2::from_u64(data, self.map_level)
    }

    //Write an entry to a location
    pub fn write_entry(&self, position: usize, entry: PageMapEntry2) -> Result<(), ReturnCode> {
        if position >= PAGE_NUMBER_1 {return Err(ReturnCode::IndexOutOfBounds)}
        if entry.entry_level != self.map_level {return Err(ReturnCode::InvalidData)}
        let data = entry.to_u64()?;
        unsafe {*((self.linear.0 as *mut u64).add(position)) = data}
        Ok(())
    }

    //Erase an entry
    pub fn erase_entry(&self, position: usize) {
        unsafe {*((self.linear.0 as *mut u64).add(position)) = 0}
    }
}

//Page Table Entry
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct PageMapEntry2 {
    pub entry_level:     PageMapLevel,
    pub entry_type:      PageMapEntryType, //Bit 7 in some cirumstances, indicates page refers to memory when it could refer to a table
    pub physical:        PhysicalAddress,
    pub present:         bool, //ALL: Bit 0, indicates entry exists
    pub write:           bool, //ALL: Bit 1, indicates page may be written to
    pub user:            bool, //ALL: Bit 2, indicates page can only be accessed in Ring 0
    pub write_through:   bool, //ALL: Bit 3, something about how memory access works
    pub cache_disable:   bool, //ALL: Bit 4, something else about how memory access works
    pub accessed:        bool, //ALL: Bit 5, indicates that a page has been accessed
    pub dirty:           Option<bool>, //MEMORY: Bit 6, indicates page has been written to
    pub attribute_table: Option<bool>, //MEMORY: Bit 7 (L1) or Bit 12 (L2, L3), indicates yet another thing about how memory access works
    pub global:          Option<bool>, //MEMORY: Bit 8,
    pub in_use:          bool, //ALL: Bit 52, indicates to the operating system that a page map entry is valid regardless of the state of the present bit
    pub execute_disable: bool, //ALL: Bit 63, indicates code may not be executed from this page
}
impl PageMapEntry2 {
    //Read from u64, intended to read a page table entry from RAM
    pub fn from_u64(data: u64, entry_level: PageMapLevel) -> Result<Self, ReturnCode> {
        let entry_type = {
            if      entry_level == PageMapLevel::L5 || entry_level == PageMapLevel::L4 {PageMapEntryType::Table}
            else if entry_level == PageMapLevel::L3 || entry_level == PageMapLevel::L2 { 
                if data & (1<<7) > 0                                                   {PageMapEntryType::Memory}
                else                                                                   {PageMapEntryType::Table}}
            else                                                                       {PageMapEntryType::Memory}
        };
        Ok(Self {
            entry_level,
            entry_type,
            physical: PhysicalAddress(match (entry_level, entry_type) {
                (PageMapLevel::L5, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L4, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L3, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L2, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L3, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_000_000_0000_u64,
                (PageMapLevel::L2, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_777_000_0000_u64,
                (PageMapLevel::L1, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_777_777_0000_u64,
                _ => {return Err(ReturnCode::InvalidData)}
            } as usize),
            present:                                                 data & (1<<0o00) > 0,
            write:                                                   data & (1<<0o01) > 0,
            user:                                                    data & (1<<0o02) > 0,
            write_through:                                           data & (1<<0o03) > 0,
            cache_disable:                                           data & (1<<0o04) > 0,
            accessed:                                                data & (1<<0o05) > 0,
            dirty: match entry_type {
                                PageMapEntryType::Memory     => Some(data & (1<<0o06) > 0),
                                PageMapEntryType::Table      => None,
            },
            attribute_table: match (entry_level, entry_type) {
                (PageMapLevel::L3, PageMapEntryType::Memory) => Some(data & (1<<0o14) > 0),
                (PageMapLevel::L2, PageMapEntryType::Memory) => Some(data & (1<<0o14) > 0),
                (PageMapLevel::L1, PageMapEntryType::Memory) => Some(data & (1<<0o07) > 0),
                _                                            => None,
            },
            global: match entry_type {
                                PageMapEntryType::Memory     => Some(data & (1<<0o10) > 0),
                                PageMapEntryType::Table      => None,
            },
            in_use:                                                  data & (1<<0o64) > 0,
            execute_disable:                                         data & (1<<0o77) > 0,
        })
    }
    
    //Convert to u64, intended to aid in writing a page table entry into RAM
    pub fn to_u64(&self) -> Result<u64, ReturnCode> {
        let mut result: u64 = 0;
        result |= match (self.entry_level, self.entry_type) {
            (PageMapLevel::L5, PageMapEntryType::Table)  => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L4, PageMapEntryType::Table)  => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L3, PageMapEntryType::Table)  => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L2, PageMapEntryType::Table)  => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L3, PageMapEntryType::Memory) => self.physical.0 as u64 & 0o_000_777_777_777_000_000_0000_u64,
            (PageMapLevel::L2, PageMapEntryType::Memory) => self.physical.0 as u64 & 0o_000_777_777_777_777_000_0000_u64,
            (PageMapLevel::L1, PageMapEntryType::Memory) => self.physical.0 as u64 & 0o_000_777_777_777_777_777_0000_u64,
            _ => {return Err(ReturnCode::InvalidData)}
        };
        if self.present       {result |= 1<<0o00}
        if self.write         {result |= 1<<0o01}
        if self.user          {result |= 1<<0o02}
        if self.write_through {result |= 1<<0o03}
        if self.cache_disable {result |= 1<<0o04}
        if self.accessed      {result |= 1<<0o05}
        if self.entry_type == PageMapEntryType::Memory {
            if self.dirty.is_some() && self.dirty.unwrap() {result |= 1<<0o06}
            if self.entry_level == PageMapLevel::L3 || self.entry_level == PageMapLevel::L2 {
                result |= 1<<0o07;
                if self.attribute_table.is_some() && self.attribute_table.unwrap() {result |= 1<<0o14}
            }
            else if self.entry_level == PageMapLevel::L1 && self.attribute_table.is_some() && self.attribute_table.unwrap() {result |= 1<<0o07}
        }
        if self.execute_disable {result |= 1<<0o77}
        Ok(result)
    }

    //New
    pub fn new(entry_level: PageMapLevel, entry_type: PageMapEntryType, address: PhysicalAddress, present: bool, write: bool, user: bool, execute_disable: bool) -> Result<Self, ReturnCode> {
        match (entry_level, entry_type) {
            (PageMapLevel::L5, PageMapEntryType::Table)  => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err(ReturnCode::UnalignedAddress)}},
            (PageMapLevel::L4, PageMapEntryType::Table)  => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err(ReturnCode::UnalignedAddress)}},
            (PageMapLevel::L3, PageMapEntryType::Table)  => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err(ReturnCode::UnalignedAddress)}},
            (PageMapLevel::L2, PageMapEntryType::Table)  => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err(ReturnCode::UnalignedAddress)}},
            (PageMapLevel::L3, PageMapEntryType::Memory) => {if address.0 as usize % PAGE_SIZE_1GIB != 0 {return Err(ReturnCode::UnalignedAddress)}},
            (PageMapLevel::L2, PageMapEntryType::Memory) => {if address.0 as usize % PAGE_SIZE_2MIB != 0 {return Err(ReturnCode::UnalignedAddress)}},
            (PageMapLevel::L1, PageMapEntryType::Memory) => {if address.0 as usize % PAGE_SIZE_4KIB != 0 {return Err(ReturnCode::UnalignedAddress)}},
            _ => {return Err(ReturnCode::InvalidData)}
        };
        Ok(Self {
            entry_level,
            entry_type,
            physical: address,
            present,
            write,
            user,
            write_through:   false,
            cache_disable:   false,
            accessed:        false,
            dirty:           if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            attribute_table: if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            global:          if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            in_use:          true,
            execute_disable,
        })
    }
}
