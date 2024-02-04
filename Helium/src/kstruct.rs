// HELIUM: KERNEL STRUCTURES
// Structs, enums, and functions which provide the methods of storing and accessing kernel structures


// HEADER
//Imports
use gluon::{x86_64::paging::{PhysicalAddress, PageMap, PageMapLevel, PageMapEntryType}, noble::data_type::DataType};


// IDENTIFIER STRUCTS
//IDs
struct ProcessID (u64);
struct ThreadID  (u64);
struct PortID    (u64);
struct TimerID   (u64);


// BASE STRUCTS
//Process
#[repr(C)]
struct Process {
    page_map_address: PhysicalAddress,
}

//Thread
#[repr(C)]
struct Thread {
    kernel_stack: usize,
}

//Port
#[repr(C)]
pub struct MemPort {
    pub address: PhysicalAddress,
    pub level: PageMapLevel,
    pub data_type: DataType,
}

pub fn port_type(level: PageMapLevel) -> PageMapEntryType {
    match level {
        PageMapLevel::L1 => PageMapEntryType::Table,
        _                => PageMapEntryType::Table,
    }
}

//Timer
#[repr(C)]
struct Timer {
    divisor: u64,
    remainder: u64,
}


//RELATIONAL
//Child Process
#[repr(C)]
struct ChildProcess {
    parent: ProcessID,
    child: ProcessID,
}

//Child Thread
#[repr(C)]
struct ChildThread {
    process: ProcessID,
    thread: ThreadID,
}

//Signaling Thread
#[repr(C)]
struct SignalingThread {
    process: ProcessID,
    thread: ThreadID,
}

//Attached Ports (Read)
#[repr(C)]
struct AttachedPortRead {
    process: ProcessID,
    port: PortID,
}

//Attached Ports (Write)
#[repr(C)]
struct AttachedPortWrite {
    process: ProcessID,
    port: PortID,
}

//Director Process
#[repr(C)]
struct DirectorProcess {
    port: PortID,
    process: ProcessID,
}

//EXECUTION CONTEXTS
//Yeilding
#[repr(C)]
struct ExecutionYeild {
    thread: ThreadID,
    timer: TimerID,
}

//Locked
#[repr(C)]
struct ExecutionLock {
    thread: ThreadID,
    port: PortID,
}

//Interrupt
struct ExecutionInterrupt {
    thread: ThreadID,
}