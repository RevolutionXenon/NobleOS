use gluon::x86_64_segmentation::*;


// SUPERVISOR CODE ENTRY
pub const SUPERVISOR_CODE_POSITION: u16 = 0x01;

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
pub const SUPERVISOR_DATA_POSITION: u16 = 0x02;

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


// USER CODE ENTRY
pub const USER_CODE_POSITION: u16 = 0x03;

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
pub const USER_DATA_POSITION: u16 = 0x04;

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


// TASK STATE SEGMENT ENTRY
pub const TASK_STATE_SEGMENT_POSITION: u16 = 0x05;

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


// TASK STATE SEGMENT
pub static mut TASK_STATE_SEGMENT: TaskStateSegment = TaskStateSegment {
    _0:    0,
    rsp0:  0,
    rsp1:  0,
    rsp2:  0,
    _1:    0,
    ist1:  0,
    ist2:  0,
    ist3:  0,
    ist4:  0,
    ist5:  0,
    ist6:  0,
    ist7:  0,
    _2:    0,
    _3:    0,
    iomba: 0,
};
