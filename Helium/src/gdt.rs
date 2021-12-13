use gluon::x86_64_segmentation::GlobalDescriptorTableEntry;


use gluon::x86_64_segmentation::*;

//SUPERVISOR CODE ENTRY
pub const SUPERVISOR_CODE_POSITION: u16 = 0x01;

pub const SUPERVISOR_CODE_ENTRY: GlobalDescriptorTableEntry = GlobalDescriptorTableEntry {
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

//SUPERVISOR DATA ENTRY
pub const SUPERVISOR_DATA_POSITION: u16 = 0x02;

pub const SUPERVISOR_DATA_ENTRY: GlobalDescriptorTableEntry = GlobalDescriptorTableEntry {
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

//USER CODE ENTRY
pub const USER_CODE_POSITION: u16 = 0x03;

pub const USER_CODE_ENTRY: GlobalDescriptorTableEntry = GlobalDescriptorTableEntry {
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

//USER DATA ENTRY
pub const USER_DATA_POSITION: u16 = 0x04;

pub const USER_DATA_ENTRY: GlobalDescriptorTableEntry = GlobalDescriptorTableEntry {
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