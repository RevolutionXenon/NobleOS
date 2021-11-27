// GLUON: IDT
// Structs and enums related to the contents and handling of x86-64 GDT and IDT structures

// HEADER
//Flags
#![allow(asm_sub_register)]

//Imports
use crate::{*, x86_64_paging::LinearAddress};


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
    //Write a GDT entry into the GDT
    pub fn write_entry(&self, entry: GlobalDescriptorTableEntry, position: u16) -> Result<(), &'static str> {
        if position >  self.limit {return Err("Global Descriptor Table: Entry position out of bounds on write.")}
        if position == 0          {return Err("Global Descriptor Table: Entry position 0 on write.")}         
        let data = entry.to_u64()?;
        unsafe {*((self.address.0 as *mut u64).add(position as usize)) = data}
        Ok(())
    }
    
    //Load the new GDT
    //This function is unsafe because doing it wrong will cause a #GP fault either immediately or during future instructions
    pub unsafe fn write_gdtr(&self, code_selector: u16, segment_selector: u16) {
        //Create byte array to load
        let mut gdt_bytes: [u8;10] = [0u8;10];
        //Place limit value into array
        let limit_bytes: [u8;2] = ((self.limit + 1) * 8 - 1).to_le_bytes();
        gdt_bytes[0x0] = limit_bytes[0x0];
        gdt_bytes[0x1] = limit_bytes[0x1];
        //Place address value into array
        let address_bytes: [u8;8] = self.address.0.to_le_bytes();
        gdt_bytes[0x2] = address_bytes[0x0];
        gdt_bytes[0x3] = address_bytes[0x1];
        gdt_bytes[0x4] = address_bytes[0x2];
        gdt_bytes[0x5] = address_bytes[0x3];
        gdt_bytes[0x6] = address_bytes[0x4];
        gdt_bytes[0x7] = address_bytes[0x5];
        gdt_bytes[0x8] = address_bytes[0x6];
        gdt_bytes[0x9] = address_bytes[0x7];
        //Load GDT and Segment Registers
        //This black magic is the way it is because interrupts will cause a #GP when they hit IRETQ without it
        asm!(
            "LGDT [{gdt}]",      //Load into GDT register from address of gdt_bytes
            "PUSH {cs}",         //Push CS value to stack
            "LEA RAX, [RIP+1f]", //Load relative address of 1: into RAX
            "PUSH RAX",          //Push RAX to stack
            "RETFQ",             //Use return to change CS register
            "1:",                //Jump here after return
            "MOV SS, {ss}",      //Change SS Register
            gdt = in(reg) &gdt_bytes,
            cs  = in(reg) code_selector,
            ss  = in(reg) segment_selector,
            options(nostack)
        );
    }
}

//GDT Entry
pub struct GlobalDescriptorTableEntry {
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
impl GlobalDescriptorTableEntry {
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


// INTERRUPT DESCRIPTOR TABLE
//IDT
#[repr(C)]
#[repr(packed)]
pub struct InterruptDescriptorTable {
    pub limit: u16,
    pub address: LinearAddress,
}
impl InterruptDescriptorTable {
    pub fn write_entry(&self, entry: &InterruptDescriptorTableEntry, position: u16) -> Result<(), &'static str> {
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
        let mut bytes: [u8;10] = [0u8;10];
        //Limit
        let limit_bytes: [u8;2] = ((self.limit+1) * 16 - 1).to_le_bytes();
        bytes[0x0] = limit_bytes[0x0];
        bytes[0x1] = limit_bytes[0x1];
        //Address
        let address_bytes: [u8;8] = self.address.0.to_le_bytes();
        bytes[0x2] = address_bytes[0x0];
        bytes[0x3] = address_bytes[0x1];
        bytes[0x4] = address_bytes[0x2];
        bytes[0x5] = address_bytes[0x3];
        bytes[0x6] = address_bytes[0x4];
        bytes[0x7] = address_bytes[0x5];
        bytes[0x8] = address_bytes[0x6];
        bytes[0x9] = address_bytes[0x7];
        //Load
        asm!("LIDT [{}]", in(reg) &bytes, options(nostack));
    }
}

//IDT Entry
pub struct InterruptDescriptorTableEntry {
    pub offset:                    u64,
    pub descriptor_table_index:    u16,
    pub table_indicator:           TableIndicator,
    pub requested_privilege_level: PrivilegeLevel,
    pub segment_present:           bool,
    pub privilege_level:           PrivilegeLevel,
    pub interrupt_stack_table:     u8,
    pub descriptor_type:           DescriptorType,
}
impl InterruptDescriptorTableEntry {
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
        //Descriptor Table Index
        let segment_selector_bytes: [u8;2] = (self.descriptor_table_index << 3).to_le_bytes();
        result[0x2] = segment_selector_bytes[0x0];
        result[0x3] = segment_selector_bytes[0x1];
        //Table Indicator
        result[0x2] |= (self.table_indicator as u8) << 2;
        //Requested Privilege Level
        result[0x2] |= self.requested_privilege_level as u8;
        //IST
        result[0x4] = self.interrupt_stack_table;
        //Flags
        result[0x5] = (self.descriptor_type as u8) | (self.privilege_level as u8) << 5 | (if self.segment_present {1} else {0}) << 7;
        //Return
        Ok(result)
    }
}

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