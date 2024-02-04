
use core::{alloc::{GlobalAlloc, Layout}, ptr::{write_volatile, read_volatile}};

use gluon::{x86_64::paging::{PageMap, LinearAddress, PageMapLevel, PAGE_SIZE_1GIB, PAGE_SIZE_4KIB}, noble::return_code::ReturnCode};

use crate::pmm::{PhysicalAddressAllocator, MemoryStack, virtual_memory_editor, PageOperation};

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum AllocState {
    Free = 0,
    LookUp = 1,
    Out = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AllocPtr {
    pub state: AllocState,
    pub next_address: usize,
}

pub struct Heap1G {
    //environment
    initialized: bool,    //Whether the allocator is ready to go
    page_map:    PageMap, //Base page map of free space (PML2)
    offset:      usize,   //Offset of memory area to be dealt with in virtual address space
    allocator:   Option<*const dyn PhysicalAddressAllocator>,
    map:         Option<*mut dyn PageOperation>,
    unmap:       Option<*mut dyn PageOperation>,

    //slab pointers (starts at 16B, ends at 1GiB)
    slab_ptr: [AllocPtr; 27],
}

//Initialization
impl Heap1G {
    pub fn new() -> Self {
        let initialized: bool = false;
        let page_map: PageMap = PageMap::new(LinearAddress(0), PageMapLevel::L2).unwrap();
        let offset: usize = 0;
        let allocator: Option<*const dyn PhysicalAddressAllocator> = Option::None;
        let map: Option<*mut dyn PageOperation> = Option::None;
        let unmap: Option<*mut dyn PageOperation> = Option::None;
        let slab_ptr: [AllocPtr; 27] = [AllocPtr{ state: AllocState::Free, next_address: 0 };27];
        Heap1G {
            initialized,
            page_map,
            offset,
            allocator,
            map,
            unmap,
            slab_ptr,
        }
    }

    pub fn init(&mut self, page_map: PageMap, offset: usize, allocator: *const dyn PhysicalAddressAllocator, map: *mut dyn PageOperation, unmap: *mut dyn PageOperation) -> Result<(), ReturnCode> {
        //Checks
        if page_map.map_level != PageMapLevel::L2 {return Err(ReturnCode::InvalidData)};
        if offset % PAGE_SIZE_1GIB != 0 {return Err(ReturnCode::InvalidData)};
        //Edit fields
        self.page_map = page_map;
        self.offset = offset;
        self.allocator = Some(allocator);
        self.map = Some(map);
        self.unmap = Some(unmap);
        //Initalize heap
        unsafe {
            virtual_memory_editor(page_map, &mut (*map), LinearAddress(0), LinearAddress(PAGE_SIZE_4KIB));
            write_volatile(offset as *mut AllocPtr, AllocPtr {state: AllocState::Out, next_address: 0});
        }
        self.slab_ptr[26] = AllocPtr {state: AllocState::Free, next_address: 0};
        //Finish
        self.initialized = true;
        Ok(())
    }
}

//Calculation
impl Heap1G {
    pub fn index_to_size(index: usize) -> usize {
        if index > 26 {panic!("heap allocator broke")}
        1 << (index + 4)
    }
    pub fn size_to_index(mut size: usize) -> usize {
        if size <= 16 {return 0}
        let mut power: usize = 2;
        size -=1;
        while size > 0 {
            size >>= 1;
            power <<= 1;
        }
        power - 4
    }
    pub fn required_index(layout: Layout) -> usize {
        if layout.align() < layout.size() {
            Self::size_to_index(layout.size())
        }
        else {
            Self::size_to_index(layout.align())
        }
    }
    pub fn split(index: usize, alloc_ptr: AllocPtr) -> Result<AllocPtr, ReturnCode> {
        match alloc_ptr.state {
            AllocState::Free => {
                let a: usize = Self::index_to_size(index + 1);
                let b: usize = Self::index_to_size(index);
                if alloc_ptr.next_address % a != 0 {Err(ReturnCode::InvalidData)}
                else {Ok(AllocPtr {state: AllocState::Free, next_address: alloc_ptr.next_address + b})
                }
            },
            AllocState::LookUp => Err(ReturnCode::InvalidData),
            AllocState::Out => Err(ReturnCode::InvalidData),
        }
    }
}

//Heap PTR ops
impl Heap1G {
    fn read_raw(&self, alloc_ptr: AllocPtr) -> Result<AllocPtr, ReturnCode> {
        match alloc_ptr.state {
            AllocState::Free   => {
                Ok(unsafe {read_volatile((self.offset + alloc_ptr.next_address) as *mut AllocPtr)})
            },
            AllocState::LookUp => Err(ReturnCode::InvalidData),
            AllocState::Out    => Err(ReturnCode::InvalidData),
        }
    }

    fn get_next(&mut self, index: usize) -> Result<AllocPtr, ReturnCode> {
        let a = self.slab_ptr[index];
        match a.state {
            AllocState::Free => {
                todo!()
            },
            AllocState::LookUp => {
                todo!()
            },
            AllocState::Out => Err(ReturnCode::OutOfResources),
        }
    }
}

//Rust global allocation
unsafe impl GlobalAlloc for Heap1G {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.initialized {
            todo!()
        }
        else {panic!("Alloc called on unitialized Heap1G.")}
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.initialized {
            todo!()
        }
        else {panic!("Dealloc called on unitialized Heap1G.")}
    }
}
