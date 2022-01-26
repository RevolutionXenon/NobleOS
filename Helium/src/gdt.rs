// HELIUM: GDT
// Consts which specify the layout and usage of the Global Descriptor Table in the Helium Kernel


// HEADER
//Imports
use gluon::x86_64::segmentation::*;


// TASK STATE SEGMENT ENTRY
pub const TASK_STATE_SEGMENT_POSITION: u16 = 0x01;

pub static mut TASK_STATE_SEGMENT_ENTRY: SystemSegmentDescriptor = SystemSegmentDescriptor {
    limit:           0x00068,
    base:            0,
    segment_type:    DescriptorType::TaskStateSegmentAvailable,
    privilege_level: PrivilegeLevel::Supervisor,
    present:         true,
    available:       true,
    granularity:     Granularity::ByteLevel,
};

pub const TASK_STATE_SEGMENT_SELECTOR: SegmentSelector = SegmentSelector {
    descriptor_table_index: TASK_STATE_SEGMENT_POSITION,
    table_indicator: TableIndicator::GDT,
    requested_privilege_level: PrivilegeLevel::Supervisor
};


// SUPERVISOR CODE ENTRY
pub const SUPERVISOR_CODE_POSITION: u16 = 0x03;

pub const SUPERVISOR_CODE_ENTRY: SegmentDescriptor = SegmentDescriptor {
    limit: 0xFFFFF,
    base: 0,
    granularity: Granularity::PageLevel,
    instruction_mode: InstructionMode::I64,
    present: true,
    privilege_level: PrivilegeLevel::Supervisor,
    segment_type: SegmentType::User,
    segment_spec: Executable::Code(Conforming::SamePrivilege, Readable::ExecuteRead),
    accessed: false,
};

pub const SUPERVISOR_CODE: SegmentSelector = SegmentSelector {
    descriptor_table_index: SUPERVISOR_CODE_POSITION,
    table_indicator: TableIndicator::GDT,
    requested_privilege_level: PrivilegeLevel::Supervisor,
};


// SUPERVISOR DATA ENTRY
pub const SUPERVISOR_DATA_POSITION: u16 = 0x04;

pub const SUPERVISOR_DATA_ENTRY: SegmentDescriptor = SegmentDescriptor {
    limit: 0xFFFFF,
    base: 0,
    granularity: Granularity::PageLevel,
    instruction_mode: InstructionMode::I64,
    present: true,
    privilege_level: PrivilegeLevel::Supervisor,
    segment_type: SegmentType::User,
    segment_spec: Executable::Data(Direction::Upwards, Writeable::ReadWrite),
    accessed: false,
};

pub const SUPERVISOR_DATA: SegmentSelector = SegmentSelector {
    descriptor_table_index: SUPERVISOR_DATA_POSITION,
    table_indicator: TableIndicator::GDT,
    requested_privilege_level: PrivilegeLevel::Supervisor,
};


// RING 1 CODE ENTRY
pub const _RING1_CODE_POSITION: u16 = 0x05;


// RING 1 DATA ENTRY
pub const _RING1_DATA_POSITION: u16 = 0x06;


// RING 2 CODE ENTRY
pub const _RING2_CODE_POSITION: u16 = 0x07;


// RING 2 DATA ENTRY
pub const _RING2_DATA_POSITION: u16 = 0x08;


// USER CODE ENTRY
pub const USER_CODE_POSITION: u16 = 0x09;

pub const USER_CODE_ENTRY: SegmentDescriptor = SegmentDescriptor {
    limit: 0xFFFFF,
    base: 0,
    granularity: Granularity::PageLevel,
    instruction_mode: InstructionMode::I64,
    present: true,
    privilege_level: PrivilegeLevel::User,
    segment_type: SegmentType::User,
    segment_spec: Executable::Code(Conforming::SamePrivilege, Readable::ExecuteRead),
    accessed: false,
};

pub const USER_CODE: SegmentSelector = SegmentSelector {
    descriptor_table_index: USER_CODE_POSITION,
    table_indicator: TableIndicator::GDT,
    requested_privilege_level: PrivilegeLevel::User,
};


// USER DATA ENTRY
pub const USER_DATA_POSITION: u16 = 0x0A;

pub const USER_DATA_ENTRY: SegmentDescriptor = SegmentDescriptor {
    limit: 0xFFFFF,
    base: 0,
    granularity: Granularity::PageLevel,
    instruction_mode: InstructionMode::I64,
    present: true,
    privilege_level: PrivilegeLevel::User,
    segment_type: SegmentType::User,
    segment_spec: Executable::Data(Direction::Upwards, Writeable::ReadWrite),
    accessed: false,
};

pub const USER_DATA: SegmentSelector = SegmentSelector {
    descriptor_table_index: USER_DATA_POSITION,
    table_indicator: TableIndicator::GDT,
    requested_privilege_level: PrivilegeLevel::User,
};
