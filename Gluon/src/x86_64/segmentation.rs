// GLUON: x86-64 SEGMENTATION
// Structs and enums related to the contents and handling of x86-64 GDT, IDT, and other segmentation structures


// HEADER
//Flags
#![allow(asm_sub_register)]

//Imports
use crate::*;
use crate::x86_64::instructions::lidt;
use crate::x86_64::instructions::ltr;
use crate::x86_64::paging::LinearAddress;
use core::arch::asm;
use core::ptr::write_volatile;


// GLOBAL DESCRIPTOR TABLE
//GDT
#[repr(C)]
#[repr(packed)]
pub struct GlobalDescriptorTable {
    pub limit: u16,
    pub address: LinearAddress,
}
impl GlobalDescriptorTable {
    //FUNCTIONS
    //Write a standard GDT entry into the GDT
    pub fn write_entry(&self, entry: SegmentDescriptor, position: u16) -> Result<(), &'static str> {
        if position >  self.limit {return Err("Global Descriptor Table: Entry position out of bounds on write.")}
        if position == 0          {return Err("Global Descriptor Table: Entry position 0 on write.")}
        let data = entry.to_u64()?;
        unsafe {*((self.address.0 as *mut u64).add(position as usize)) = data}
        Ok(())
    }

    //Write a system GDT entry into the GDT
    pub fn write_system_entry(&self, entry: SystemSegmentDescriptor, position: u16) -> Result<(), &'static str> {
        if position >  self.limit {return Err("Global Descriptor Table: Entry position out of bounds on write.")}
        if position == 0          {return Err("Global Descriptor Table: Entry position 0 on write.")}
        let data = entry.to_bytes()?;
        unsafe {
            let address = (self.address.0 as *mut u64).add(position as usize) as *mut u8;
            for (i, byte) in data.iter().enumerate() {
                write_volatile(address.add(i), *byte);
            }
        }
        Ok(())
    }
    
    //Load the new GDT
    //This function is unsafe because doing it wrong will cause a #GP fault either immediately or during future instructions
    pub unsafe fn write_gdtr(&self, code_selector: SegmentSelector, data_selector: SegmentSelector, stack_selector: SegmentSelector) {
        //Create byte array to load
        let mut gdtr: [u8;10] = [0u8;10];
        //Place limit value into array
        let limit_bytes: [u8;2] = ((self.limit + 1) * 8 - 1).to_le_bytes();
        gdtr[0x0] = limit_bytes[0x0];
        gdtr[0x1] = limit_bytes[0x1];
        //Place address value into array
        let address_bytes: [u8;8] = self.address.0.to_le_bytes();
        gdtr[0x2] = address_bytes[0x0];
        gdtr[0x3] = address_bytes[0x1];
        gdtr[0x4] = address_bytes[0x2];
        gdtr[0x5] = address_bytes[0x3];
        gdtr[0x6] = address_bytes[0x4];
        gdtr[0x7] = address_bytes[0x5];
        gdtr[0x8] = address_bytes[0x6];
        gdtr[0x9] = address_bytes[0x7];
        //Load GDT and Segment Registers
        //This black magic is the way it is because interrupts will cause a #GP when they hit IRETQ without it
        asm!(
            "LGDT [{gdt}]",      //Load into GDT register from address of gdtr
            "PUSH {cs}",         //Push CS value to stack
            "LEA RAX, [RIP+1f]", //Load relative address of 1: into RAX
            "PUSH RAX",          //Push RAX to stack
            "RETFQ",             //Use return to change Code Segment register
            "1:",                //Jump here after return
            "MOV SS, {ss}",      //Change Stack Segment register
            "MOV DS, {ds}",      //Change Data Segment register
            gdt = in(reg) &gdtr,
            cs  = in(reg) u16::from(code_selector),
            ds  = in(reg) u16::from(data_selector),
            ss  = in(reg) u16::from(stack_selector),
            options(nostack)
        );
    }
}

//Standard GDT Entry
#[derive(Clone, Copy)]
pub struct SegmentDescriptor {
    pub limit:            u32,
    pub base:             u32,
    pub granularity:      Granularity,
    pub instruction_mode: InstructionMode,
    pub present:          bool,
    pub privilege_level:  PrivilegeLevel,
    pub segment_type:     SegmentType,
    pub segment_spec:     Executable,
    pub accessed:         bool,
}
impl SegmentDescriptor {
    pub fn to_u64(&self) -> Result<u64, &'static str> {
        if self.limit >= (1<<20) {return Err("Global Descriptor Table Entry: Limit exceeds bounds.")}
        let mut result: u64 = 0;
        //Base
        let base_bytes: [u8; 4] = self.base.to_le_bytes();
        result |= (base_bytes[3] as u64) << 56;
        result |= (base_bytes[2] as u64) << 32;
        result |= (base_bytes[1] as u64) << 24;
        result |= (base_bytes[0] as u64) << 16;
        //Limit
        let limit_bytes: [u8; 4] = self.limit.to_le_bytes();
        result |= (limit_bytes[2] as u64) << 48;
        result |= (limit_bytes[1] as u64) <<  8;
        result |=  limit_bytes[0] as u64;
        //Flags
        result |= (self.granularity      as u64) << (52 + 3);
        result |= (self.instruction_mode as u64) << (52 + 1);
        //Access Byte
        result |= (self.present          as u64) << (40 + 7);
        result |= (self.privilege_level  as u64) << (40 + 5);
        result |= (self.segment_type     as u64) << (40 + 4);
        match self.segment_spec {
            Executable::Data(direction, writeable) => {
                //result |= 0 << (40 + 3);
                result |= (direction  as u64) << (40 + 2);
                result |= (writeable  as u64) << (40 + 1);
            },
            Executable::Code(conforming, readable) => {
                result |= 1 << (40 + 3);
                result |= (conforming as u64) << (40 + 2);
                result |= (readable   as u64) << (40 + 1);
            },
        }
        result |= (self.accessed as u64) << 40;
        //Return
        Ok(result)
    }
}

//System GDT Entry
#[derive(Clone, Copy)]
pub struct SystemSegmentDescriptor {
    pub limit:            u32,
    pub base:             u64,
    pub segment_type:     DescriptorType,
    pub privilege_level:  PrivilegeLevel,
    pub present:          bool,
    pub available:        bool,
    pub granularity:      Granularity,
}
impl SystemSegmentDescriptor {
    pub fn to_bytes(&self) -> Result<[u8;16], &'static str> {
        if self.limit >= (1<<20) {return Err("Global Descriptor Table Entry: Limit exceeds bounds.")}
        let mut result: [u8;16] = [0;16];
        //Base
        let base_bytes: [u8; 8] = self.base.to_le_bytes();
        result[0x2]  = base_bytes[0x0];
        result[0x3]  = base_bytes[0x1];
        result[0x4]  = base_bytes[0x2];
        result[0x7]  = base_bytes[0x3];
        result[0x8]  = base_bytes[0x4];
        result[0x9]  = base_bytes[0x5];
        result[0xA]  = base_bytes[0x6];
        result[0xB]  = base_bytes[0x7];
        //Limit
        let limit_bytes: [u8; 4] = self.limit.to_le_bytes();
        result[0x0]  = limit_bytes[0x0];
        result[0x1]  = limit_bytes[0x1];
        result[0x6]  = limit_bytes[0x2];
        //Flags
        result[0x6] |= (self.granularity as u8) << 7;
        result[0x6] |= (self.available   as u8) << 4;
        //Access Byte
        result[0x5]  =  self.segment_type    as u8;
        result[0x5] |= (self.privilege_level as u8) << 5;
        result[0x5] |= (self.present         as u8) << 7;
        //Return
        Ok(result)
    }
}


// SEGMENT SELECTOR
//Selector
#[derive(Clone, Copy)]
pub struct SegmentSelector {
    pub descriptor_table_index :   u16,
    pub table_indicator:           TableIndicator,
    pub requested_privilege_level: PrivilegeLevel,
}
impl SegmentSelector {
    pub fn to_bytes(&self) -> Result<[u8;2], &'static str> {
        if self.descriptor_table_index >= 0x2000 {return Err("Segment Selector: Entry position out of bounds.")}
        let mut result: [u8;2] = [0u8;2];
        //Descriptor Table Index
        let index_bytes: [u8;2] = (self.descriptor_table_index << 3).to_le_bytes();
        result[0x0] = index_bytes[0x0];
        result[0x1] = index_bytes[0x1];
        //Table Indicator
        result[0x0] |= (self.table_indicator as u8) << 2;
        //Requested Privilege Level
        result[0x0] |= self.requested_privilege_level as u8;
        //Return
        Ok(result)
    }
}
impl From<SegmentSelector> for u16 {
    fn from(segment_selector: SegmentSelector) -> Self {
        (segment_selector.descriptor_table_index << 3) | ((segment_selector.table_indicator as u16) << 2) | (segment_selector.requested_privilege_level as u16)
    }
}
impl From<u16> for SegmentSelector {
    fn from(raw: u16) -> Self {
        Self {
            descriptor_table_index: raw >> 3,
            table_indicator: TableIndicator::try_from(((raw as u8) & 0x4) >> 2).unwrap(),
            requested_privilege_level: PrivilegeLevel::try_from(raw as u8 & 0x3).unwrap(),
        }
    }
}


// TASK STATE SEGMENT
//TSS
#[repr(C)]
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct TaskStateSegment {
    pub _0:    u32,
    pub rsp0:  u64,
    pub rsp1:  u64,
    pub rsp2:  u64,
    pub _1:    u64,
    pub ist1:  u64,
    pub ist2:  u64,
    pub ist3:  u64,
    pub ist4:  u64,
    pub ist5:  u64,
    pub ist6:  u64,
    pub ist7:  u64,
    pub _2:    u64,
    pub _3:    u16,
    pub iomba: u16,
}

//Load Task Register
pub fn load_task_register(selector: SegmentSelector) {
    ltr(u16::from(selector))
}


// INTERRUPT DESCRIPTOR TABLE
//IDT
#[repr(C)]
#[repr(packed)]
pub struct InterruptDescriptorTable {
    pub limit: u16,
    pub address: LinearAddress,
}
impl InterruptDescriptorTable {
    pub fn write_entry(&self, entry: &InterruptDescriptor, position: u16) -> Result<(), &'static str> {
        if position >  self.limit {return Err("Interrupt Descriptor Table: Entry position out of bounds on write.")}
        let data = entry.to_bytes()?;
        for (i, byte) in data.iter().enumerate().take(12) {
            unsafe {*((self.address.0) as *mut u8).add((position as usize)*16).add(i) = *byte}
        }
        Ok(())
    }

    pub fn read_entry_raw(&self, position: u16) -> Result<[u8;16], &'static str> {
        if position > self.limit {return Err("Interrupt Descriptor Table: Entry position out of bounds on raw read.")}
        let mut result: [u8;16] = [0u8;16];
        for (i, byte) in result.iter_mut().enumerate() {
            *byte = unsafe {*((self.address.0) as *mut u8).add(position as usize * 16).add(i)}
        }
        Ok(result)
    }

    pub unsafe fn write_idtr(&self) {
        let mut idtr: [u8;10] = [0u8;10];
        //Limit
        let limit_bytes: [u8;2] = ((self.limit+1) * 16 - 1).to_le_bytes();
        idtr[0x0] = limit_bytes[0x0];
        idtr[0x1] = limit_bytes[0x1];
        //Address
        let address_bytes: [u8;8] = self.address.0.to_le_bytes();
        idtr[0x2] = address_bytes[0x0];
        idtr[0x3] = address_bytes[0x1];
        idtr[0x4] = address_bytes[0x2];
        idtr[0x5] = address_bytes[0x3];
        idtr[0x6] = address_bytes[0x4];
        idtr[0x7] = address_bytes[0x5];
        idtr[0x8] = address_bytes[0x6];
        idtr[0x9] = address_bytes[0x7];
        //Load
        lidt(&idtr);
    }
}

//IDT Entry
#[derive(Clone, Copy)]
pub struct InterruptDescriptor {
    pub offset:                u64,
    pub segment_selector:      SegmentSelector,
    pub segment_present:       bool,
    pub privilege_level:       PrivilegeLevel,
    pub interrupt_stack_table: u8,
    pub descriptor_type:       DescriptorType,
}
impl InterruptDescriptor {
    pub fn to_bytes(&self) -> Result<[u8;16], &'static str> {
        if self.interrupt_stack_table > 0x7 {return Err("Interrupt Descriptor Table Entry: IST out of bounds.")}
        let mut result: [u8;16] = [0u8;16];
        //Offset
        let offset_bytes: [u8;8] = self.offset.to_le_bytes();
        result[0x0] = offset_bytes[0x0];
        result[0x1] = offset_bytes[0x1];
        result[0x6] = offset_bytes[0x2];
        result[0x7] = offset_bytes[0x3];
        result[0x8] = offset_bytes[0x4];
        result[0x9] = offset_bytes[0x5];
        result[0xA] = offset_bytes[0x6];
        result[0xB] = offset_bytes[0x7];
        //Segment Selector
        let segment_selector_bytes: [u8;2] = self.segment_selector.to_bytes()?;
        result[0x2] = segment_selector_bytes[0];
        result[0x3] = segment_selector_bytes[1];
        //IST
        result[0x4] = self.interrupt_stack_table;
        //Flags
        result[0x5] = (self.descriptor_type as u8) | (self.privilege_level as u8) << 5 | (if self.segment_present {1} else {0}) << 7;
        //Return
        Ok(result)
    }
}


// INTERRUPT STACK FRAME
//ISF
#[repr(C)]
pub struct InterruptStackFrame {
    rip:    u64,
    cs:     u64,
    rflags: u64,
    rsp:    u64,
    ss:     u64,
}
impl InterruptStackFrame {
    //Constructor
    pub fn new(code_pointer: LinearAddress, stack_pointer: LinearAddress, code_selector: SegmentSelector, stack_selector: SegmentSelector, rflags_image: u64) -> Self {
        Self {
            rip:    code_pointer.0 as u64,
            cs:     u16::from(code_selector) as u64,
            rflags: rflags_image,
            rsp:    stack_pointer.0 as u64,
            ss:     u16::from(stack_selector) as u64,
        }
    }

    //Retrieval
    pub fn code_pointer(&self)   -> LinearAddress   {LinearAddress(self.rip as usize)}
    pub fn stack_pointer(&self)  -> LinearAddress   {LinearAddress(self.rsp as usize)}
    pub fn code_selector(&self)  -> SegmentSelector {SegmentSelector::from(self.cs as u16)}
    pub fn stack_selector(&self) -> SegmentSelector {SegmentSelector::from(self.ss as u16)}
    pub fn rflags_image(&self)   -> u64             {self.rflags}
}


// ENUMS
//Descriptor Types
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum DescriptorType {
        LocalDescriptorTable      = 0x2,
        TaskStateSegmentAvailable = 0x9,
        TaskStateSegmentBusy      = 0xB,
        CallGate                  = 0xC,
        InterruptGate             = 0xE,
        TrapGate                  = 0xF,
    }
}

//Table Indicator
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum TableIndicator {
        GDT = 0x0,
        LDT = 0x1,
    }
}

//Granularity
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum Granularity {
        ByteLevel = 0x0,
        PageLevel = 0x1,
    }
}

//Instruction Mode
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum InstructionMode {
        I16 = 0x0,
        I32 = 0x2,
        I64 = 0x1,
    }
}

//CPU Ring
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum PrivilegeLevel {
        Supervisor = 0x0,
        Ring1      = 0x1,
        Ring2      = 0x2,
        User       = 0x3,
    }
}

//Segment Type
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum SegmentType {
        System = 0x0,
        User = 0x1,
    }
}

//Executable
#[repr(u8)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum Executable {
    Data (Direction, Writeable) = 0x0,
    Code (Conforming, Readable) = 0x1,
}

//Direction
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum Direction {
        Upwards = 0x0,
        Downwards = 0x1,
    }
}

//Conforming
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum Conforming {
        SamePrivilege = 0x0,
        LessPrivilege = 0x1,
    }
}

//Writeable
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum Writeable {
        ReadOnly = 0x0,
        ReadWrite = 0x1,
    }
}

//Readable
numeric_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum Readable {
        ExecuteOnly = 0x0,
        ExecuteRead = 0x1,
    }
}
