// HELIUM: PHYSICAL MEMORY MANAGMENT


// HEADER
//Imports
use core::cell::RefCell;
use core::ptr::read_volatile;
use core::ptr::write_volatile;
use core::intrinsics::write_bytes;
use gluon::x86_64::paging::*;
use gluon::noble::return_code::*;


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


// ADDRESS TRANSLATION
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
        Ok(LinearAddress(physical.0 + self.offset))
    }
}


// PHYSICAL ADDRESS ALLOCATION
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

//Iterator Allocator
pub struct IteratorAllocator<'s> {
    pub iter_ref: RefCell<&'s mut dyn Iterator<Item = PhysicalAddress>>,
}
impl<'i> PhysicalAddressAllocator for IteratorAllocator<'i> {
    fn take(&self, pages: &mut [PhysicalAddress]) -> Result<(), ReturnCode> {
        for i in pages {
            *i = match self.iter_ref.borrow_mut().next() {
                Some(e) => e,
                None => {return Err(ReturnCode::OutOfResources)},
            };
        }
        Ok(())
    }
    fn give(&self, _pages: &[PhysicalAddress]) -> Result<(), ReturnCode> {
        Err(ReturnCode::UnsupportedFeature)
    }
}

//Stack Allocator
pub struct MemoryStack<'s> {
    pub index: RefCell<usize>,
    pub stack: *const PhysicalAddress,
    pub translator: &'s dyn AddressTranslator,
}
impl<'i> PhysicalAddressAllocator for MemoryStack<'i> {
    fn take(&self, pages: &mut [PhysicalAddress]) -> Result<(), ReturnCode> {
        //CRITICAL SECTION
        {
            if pages.len() > *self.index.borrow() {return Err(ReturnCode::OutOfResources)}
            let old_index: usize = *self.index.borrow(); //unsafe {read_volatile(self.index)};
            let new_index: usize = old_index - pages.len();
            if old_index < pages.len() {return Err(ReturnCode::OutOfResources)}
            *self.index.borrow_mut() = new_index; //unsafe {write_volatile(self.index, new_index)};
            unsafe {core::ptr::copy_nonoverlapping(self.stack.add(new_index), pages.as_mut_ptr(), pages.len())};
        }
        //Zero memory
        for physical in pages {
            let linear = self.translator.translate(*physical)?;
            unsafe {write_bytes(linear.0 as *mut u8, 0, PAGE_SIZE_4KIB)}
        }
        Ok(())
    }

    fn give(&self, pages: &[PhysicalAddress]) -> Result<(), ReturnCode> {
        //CRITICAL SECTION
        {
            let old_index: usize = *self.index.borrow(); //unsafe {read_volatile(self.index)};
            let new_index: usize = old_index + pages.len();
            unsafe {core::ptr::copy_nonoverlapping(pages.as_ptr(), self.stack.add(old_index) as *mut PhysicalAddress, pages.len())};
            *self.index.borrow_mut() = new_index; //unsafe {write_volatile(self.index, new_index)}
        }
        Ok(())
    }
}


// PAGE OPERATIONS
//Page Operation Trait
pub trait PageOperation {
    fn op(&mut self, entry: PageMapEntry, start: LinearAddress, end: LinearAddress) -> Result<PageMapEntry, ReturnCode>;
}

//Map Memory
pub struct MapMemory<'s> {
    pub allocator: &'s dyn PhysicalAddressAllocator,
    pub translator: &'s dyn AddressTranslator,
    pub write: bool,
    pub user: bool,
    pub execute_disable: bool,
}
impl<'i> PageOperation for MapMemory<'i> {
    fn op(&mut self, entry: PageMapEntry, start: LinearAddress, end: LinearAddress) -> Result<PageMapEntry, ReturnCode> {
        match (entry.in_use, entry.entry_level, entry.entry_type) {
            (true,  _, PageMapEntryType::Memory) => {Err(ReturnCode::Test03)}, //throw error due to previously allocated memory
            (true,  _, PageMapEntryType::Table)  => {
                //use existing table and recurse
                let map = PageMap::new(self.translator.translate(entry.physical)?, entry.entry_level.sub()?)?;
                virtual_memory_editor(map, self, start, end)?;
                //writeln!(self.printer, "tlb: {:?}", entry);
                Ok(entry)
            },
            (false, PageMapLevel::L1, _) => {
                //allocate a single page as memory
                let address = self.allocator.take_one()?;
                let value = PageMapEntry::new(PageMapLevel::L1, PageMapEntryType::Memory, address, true, self.write, self.user, self.execute_disable);
                //writeln!(self.printer, "nwm: {:?}", value);
                value
            },
            (false, _, _) => {
                //allocate a single page as a table and recurse
                let physical = self.allocator.take_one()?;
                let linear = self.translator.translate(physical)?;
                let map = PageMap::new(linear, entry.entry_level.sub()?)?;
                virtual_memory_editor(map, self, start, end)?;
                let value = PageMapEntry::new(entry.entry_level, PageMapEntryType::Table, physical, true, self.write, self.user, self.execute_disable);
                //writeln!(self.printer, "nwt: {:?}", value);
                value
            },
        }
    }
}

//Unmap Memory
pub struct UnmapMemory<'s> {
    pub allocator: &'s dyn PhysicalAddressAllocator,
    pub translator: &'s dyn AddressTranslator,
}
impl<'i> PageOperation for UnmapMemory<'i> {
    fn op(&mut self, entry: PageMapEntry, start: LinearAddress, end: LinearAddress) -> Result<PageMapEntry, ReturnCode> {
        match (entry.in_use, entry.entry_type, entry.entry_level) {
            (false, _, _) => Err(ReturnCode::Test05), //throw error due to deallocating area not in use
            (true, PageMapEntryType::Table, _) => {
                //recurse through existing table
                //writeln!(self.printer, "traverse: {:?}", entry.physical);
                let map = PageMap::new(self.translator.translate(entry.physical)?, entry.entry_level.sub()?)?;
                virtual_memory_editor(map, self, start, end)?;
                //test if map is empty
                let mut map_empty: bool = true;
                for position in 0usize..512 {
                    let map_entry = map.read_entry(position)?;
                    if map_entry.in_use {
                        map_empty = false;
                        break
                    }
                }
                //finish
                //return Ok(entry); // PROBLEM SOMEWHERE HERE
                if map_empty {
                    let physical = entry.physical;
                    self.allocator.give(&[physical])?;
                    PageMapEntry::from_u64(0, entry.entry_level)
                }
                else {
                    Ok(entry)
                }
            },
            (true, PageMapEntryType::Memory, PageMapLevel::L1) => {
                //deallocate 4KB memory block
                let physical = entry.physical;
                //writeln!(self.printer, "dealloc: {:?}", physical);
                self.allocator.give(&[physical])?;
                PageMapEntry::from_u64(0, PageMapLevel::L1)
            },
            (true, PageMapEntryType::Memory, _) => Err(ReturnCode::Test04), //throw error due to deallocating non-4KB memory block
        }
    }
}

//Mark in use
pub struct MarkInUse<'s> {
    pub translator: &'s dyn AddressTranslator,
}
impl<'i> PageOperation for MarkInUse<'i> {
    fn op(&mut self, mut entry: PageMapEntry, start: LinearAddress, end: LinearAddress) -> Result<PageMapEntry, ReturnCode> {
        match (entry.in_use, entry.present, entry.entry_type) {
            (true,  _,     _) => {},
            (_,     false, _) => {},
            (false, true,  PageMapEntryType::Memory) => {
                entry.in_use = true;
            },
            (false, true,  PageMapEntryType::Table) => {
                let map = PageMap::new(self.translator.translate(entry.physical)?, entry.entry_level.sub()?)?;
                virtual_memory_editor(map, self, start, end)?;
                entry.in_use = true;
            },
        }
        Ok(entry)
    }
}

//Deprivilege
pub struct DePrivilege<'s> {
    pub translator: &'s dyn AddressTranslator,
}
impl<'i> PageOperation for DePrivilege<'i> {
    fn op(&mut self, mut entry: PageMapEntry, start: LinearAddress, end: LinearAddress) -> Result<PageMapEntry, ReturnCode> {
        match (entry.in_use, entry.entry_type) {
            (true, PageMapEntryType::Memory) => {
                entry.user = true;
            },
            (true, PageMapEntryType::Table) => {
                let map = PageMap::new(self.translator.translate(entry.physical)?, entry.entry_level.sub()?)?;
                virtual_memory_editor(map, self, start, end)?;
                entry.user = true;
            },
            (_, _) => {},
        }
        Ok(entry)
    }
}

// PAGE OPERATION
//Virtual Memory Editor
pub fn virtual_memory_editor(map: PageMap, operation: &mut dyn PageOperation, start: LinearAddress, end: LinearAddress) -> Result<(), ReturnCode> {
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
        let entry_read: PageMapEntry = map.read_entry(index_current)?;
        let entry_write: PageMapEntry = operation.op(entry_read, start_current, end_current)?;
        map.write_entry(index_current, entry_write)?;
        //
        start_current = end_current;
    }
    //
    Ok(())
}
