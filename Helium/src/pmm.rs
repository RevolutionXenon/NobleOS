// HELIUM: PHYSICAL MEMORY MANAGER

use core::{ptr::{read_volatile, write_volatile}, fmt::Write, intrinsics::{copy_nonoverlapping, write_bytes}};

use gluon::{x86_64::paging::{LinearAddress, PhysicalAddress, PageAllocator, PAGE_SIZE_4KIB, canonical_48, extract_index, page_size, PageMap2, PageMapEntry2, PageMapEntryType, PageMapLevel}, noble::return_code::ReturnCode};

// MEMORY MANAGEMENT
//Address translator which cannot allocate
pub struct NoneAllocator {
    pub identity_offset: usize
}
impl PageAllocator for NoneAllocator {
    fn allocate_page     (&self)                            -> Result<PhysicalAddress, &'static str> {
        Err("No Allocator: Allocate page called.")
    }
    fn deallocate_page   (&self, _physical: PhysicalAddress) -> Result<(),              &'static str> {
        Err("No Allocator: De-allocate page called.")
    }
    fn physical_to_linear(&self, physical: PhysicalAddress) -> Result<LinearAddress,   &'static str> {
        Ok(LinearAddress(physical.add(self.identity_offset).0))
    }
}

//Stack allocator handed over from bootloader
pub struct StackAllocator {
    pub position:    *mut usize,
    pub base_offset: *mut u64,
    pub identity_offset:  usize,
}
impl PageAllocator for StackAllocator {
    fn allocate_page     (&self)                            -> Result<PhysicalAddress, &'static str> { unsafe {
        match read_volatile(self.position) {
            0 => Err("Stack Page Allocator: Out of memory."),
            position => Ok(PhysicalAddress({
                write_volatile(self.position, position-1);
                let address = read_volatile(self.base_offset.add(position-1)) as usize;
                let clear_pointer = (address + self.identity_offset) as *mut u8;
                for i in 0..PAGE_SIZE_4KIB {write_volatile(clear_pointer.add(i), 0);}
                address
            }))
        }
    }}

    fn deallocate_page   (&self, physical: PhysicalAddress) -> Result<(),              &'static str> {unsafe {
        write_volatile(self.base_offset.add(*self.position), physical.0 as u64);
        *self.position += 1;
        Ok(())
    }}

    fn physical_to_linear(&self, physical: PhysicalAddress) -> Result<LinearAddress,   &'static str> {
        Ok(LinearAddress(physical.add(self.identity_offset).0))
    }
}


// NEW SYSTEM
//Address Translator Trait
pub trait AddressTranslator {
    fn translate(&self, physical: PhysicalAddress) -> Result<LinearAddress, ReturnCode>;
}

//Offset Identity Address Translator
pub struct OffsetIdentity {
    pub offset: usize,
    pub limit: usize,
}
impl AddressTranslator for OffsetIdentity {
    fn translate(&self, physical: PhysicalAddress) -> Result<LinearAddress, ReturnCode> {
        if physical.0 > self.limit {return Err(ReturnCode::MemoryOutOfBounds)}
        else {Ok(LinearAddress(physical.0 + self.offset))}
    }
}

//Physical Allocator Trait
pub trait PhysicalAddressAllocator {
    fn take(&self, pages: &mut [PhysicalAddress]) -> Result<(), ReturnCode>;
    fn give(&self, pages: &[PhysicalAddress]) -> Result<(), ReturnCode>;
    fn take_one(&self) -> Result<PhysicalAddress, ReturnCode> {
        let mut buffer: [PhysicalAddress; 1] = [PhysicalAddress(0)];
        self.take(&mut buffer)?;
        Ok(buffer[0])
    }
    fn give_one(&self, page: PhysicalAddress) -> Result<(), ReturnCode> {
        let array: [PhysicalAddress; 1] = [page];
        self.give(&array)
    }
}

//Stack Allocator
pub struct MemoryStack<'s> {
    pub index: *mut usize,
    pub stack: *const PhysicalAddress,
    pub translator: &'s dyn AddressTranslator,
}
impl<'i> PhysicalAddressAllocator for MemoryStack<'i> {
    fn take(&self, pages: &mut [PhysicalAddress]) -> Result<(), ReturnCode> {
        //CRITICAL SECTION
        {
            let old_index: usize = unsafe {read_volatile(self.index)};
            if old_index < pages.len() {return Err(ReturnCode::OutOfResources)}
            let new_index = old_index - pages.len();
            unsafe {write_volatile(self.index, new_index)};
            unsafe {core::ptr::copy_nonoverlapping(self.stack.add(new_index), pages.as_mut_ptr(), pages.len())};
        }
        //Zero memory
        for physical in pages {
            let linear = self.translator.translate(*physical)?;
            unsafe {write_bytes(linear.0 as *mut u8, 0, 4096)}
        }
        Ok(())
    }

    fn give(&self, pages: &[PhysicalAddress]) -> Result<(), ReturnCode> {
        todo!()
    }
}

//Page Operation Trait
pub trait PageOperation {
    fn op(&mut self, entry: PageMapEntry2, start: LinearAddress, end: LinearAddress) -> Result<PageMapEntry2, ReturnCode>;
}

//Map Memory
pub struct MapMemory<'s> {
    pub allocator: &'s dyn PhysicalAddressAllocator,
    pub translator: &'s dyn AddressTranslator,
    pub write: bool,
    pub user: bool,
    pub execute_disable: bool,
    pub printer: &'s mut dyn Write,
}
impl<'i> PageOperation for MapMemory<'i> {
    fn op(&mut self, entry: PageMapEntry2, start: LinearAddress, end: LinearAddress) -> Result<PageMapEntry2, ReturnCode> {
        match (entry.in_use, entry.entry_level, entry.entry_type) {
            (true,  _, PageMapEntryType::Memory) => {Err(ReturnCode::Test03)}, //throw error due to previously allocated memory
            (true,  _, PageMapEntryType::Table)  => {
                //use existing table and recurse
                let map = PageMap2::new(self.translator.translate(entry.physical)?, entry.entry_level.sub()?)?;
                virtual_memory_editor(map, self, start, end)?;
                writeln!(self.printer, "tlb: {:?}", entry);
                Ok(entry)
            },
            (false, PageMapLevel::L1, _) => {
                //allocate a single page as memory
                let address = self.allocator.take_one()?;
                let value = PageMapEntry2::new(PageMapLevel::L1, PageMapEntryType::Memory, address, true, self.write, self.user, self.execute_disable);
                writeln!(self.printer, "nwm: {:?}", value);
                value
            },
            (false, _, _) => {
                //allocate a single page as a table and recurse
                let physical = self.allocator.take_one()?;
                let linear = self.translator.translate(physical)?;
                let map = PageMap2::new(linear, entry.entry_level.sub()?)?;
                virtual_memory_editor(map, self, start, end)?;
                let value = PageMapEntry2::new(entry.entry_level, PageMapEntryType::Table, physical, true, self.write, self.user, self.execute_disable);
                writeln!(self.printer, "nwt: {:?}", value);
                value
            },
        }
    }
}

//Virtual Memory Editor
pub fn virtual_memory_editor(map: PageMap2, operation: &mut dyn PageOperation, start: LinearAddress, end: LinearAddress) -> Result<(), ReturnCode> {
    //
    canonical_48(start)?; canonical_48(end)?;
    if end.0 <= start.0 {return Err(ReturnCode::Test01)}
    //
    let index_start: usize = extract_index(start, map.map_level);
    let index_end: usize = extract_index(end.sub(1), map.map_level);
    let mut start_current: LinearAddress = start;
    let p = page_size(map.map_level);
    //
    for index_current in index_start..index_end+1 {
        //
        let end_current: LinearAddress =
            if index_current == index_end {end}
            else {LinearAddress(((start_current.0 + p)/p) * p)};
        //
        let entry_read: PageMapEntry2 = map.read_entry(index_current)?;
        let entry_write: PageMapEntry2 = operation.op(entry_read, start_current, end_current)?;
        map.write_entry(index_current, entry_write)?;
        //
        start_current = end_current;
    }
    //
    Ok(())
}
