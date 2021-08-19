// GLUON
// Gluon is the Noble loading library:
// Memory locations of important objects
// Sizes and counts related to page tables
// Macros, Structs, and Enums related to the contents and handling of ELF files


// HEADER
//Flags
#![no_std]

//Imports
use core::convert::TryFrom;
use core::intrinsics::copy_nonoverlapping;
use header::Header;
use header::ObjectType;
use program::ProgramIterator;
use program::ProgramType;
use program_dynamic_entry::ProgramDynamicEntryIterator;
use program_dynamic_entry::ProgramDynamicEntryType;
use section::SectionIterator;

//Constants
pub const GLUON_VERSION:  &    str   = "vDEV-2021-08-18";                                                //CURRENT VERSION OF GRAPHICS LIBRARY
//                                                         SIGN PM5 PM4 PM3 PM2 PM1 OFFSET
pub const PHYSICAL_MEMORY_PHYSICAL_OCTAL:        usize = 0o_________000__________________usize;          //PHYSICAL MEMORY PHYSICAL LOCATION PML4 OFFSET
pub const PHYSICAL_MEMORY_PHYSICAL_POINTER: *mut u8    = 0o_000_000_000_000_000_000_0000_u64 as *mut u8; //PHYSICAL MEMORY PHYSICAL LOCATION POINTER
pub const KERNEL_VIRTUAL_OCTAL:                  usize = 0o_________400__________________usize;          //KERNEL VIRTUAL LOCATION PML4 TABLE OFFSET
pub const KERNEL_VIRTUAL_POINTER:           *mut u8    = 0o_177_777_400_000_000_000_0000_u64 as *mut u8; //KERNEL VIRTUAL LOCATION POINTER
pub const FRAME_BUFFER_VIRTUAL_OCTAL:            usize = 0o_________775__________________usize;          //FRAME BUFFER VIRTUAL LOCATION PML4 OFFSET
pub const FRAME_BUFFER_VIRTUAL_POINTER:     *mut u8    = 0o_177_777_775_000_000_000_0000_u64 as *mut u8; //FRAME BUFFER VIRTUAL LOCATION POINTER
pub const PHYSICAL_MEMORY_VIRTUAL_OCTAL:         usize = 0o_________776__________________usize;          //PHYSICAL MEMORY VIRTUAL LOCATION PML4 OFFSET
pub const PHYSICAL_MEMORY_VIRTUAL_POINTER:  *mut u8    = 0o_177_777_776_000_000_000_0000_u64 as *mut u8; //PHYSICAL MEMORY VIRTUAL LOCATION POINTER
pub const PAGE_MAP_VIRTUAL_OCTAL:                usize = 0o_________777__________________usize;          //PAGE MAP VIRTUAL LOCATION PML4 OFFSET
pub const PAGE_MAP_VIRTUAL_POINTER:         *mut u8    = 0o_177_777_777_000_000_000_0000_u64 as *mut u8; //PAGE MAP VIRTUAL LOCATION POINTER
pub const PAGE_SIZE_4KIB:                        usize = 0o_______________________1_0000_usize;          //MEMORY PAGE SIZE (  4KiB),                            PAGE MAP LEVEL 1 ENTRY SIZE
pub const PAGE_SIZE_2MIB:                        usize = 0o___________________1_000_0000_usize;          //MEMORY PAGE SIZE (  2MiB), PAGE MAP LEVEL 1 CAPACITY, PAGE MAP LEVEL 2 ENTRY SIZE
pub const PAGE_SIZE_1GIB:                        usize = 0o_______________1_000_000_0000_usize;          //MEMORY PAGE SIZE (  1GiB), PAGE MAP LEVEL 2 CAPACITY, PAGE MAP LEVEL 3 ENTRY SIZE
pub const PAGE_SIZE_512G:                        usize = 0o___________1_000_000_000_0000_usize;          //MEMORY PAGE SIZE (512GiB), PAGE MAP LEVEL 3 CAPACITY
pub const PAGE_SIZE_256T:                        usize = 0o_______1_000_000_000_000_0000_usize;          //MEMORY PAGE SIZE (256TiB), PAGE MAP LEVEL 4 CAPACITY
pub const PAGE_SIZE_128P:                        usize = 0o___1_000_000_000_000_000_0000_usize;          //MEMORY PAGE SIZE (128PiB), PAGE MAP LEVEL 5 CAPACITY
pub const PAGE_NUMBER_1:                         usize = 0o_________________________1000_usize;          //NUMBER OF PAGE TABLE ENTRIES 1 LEVELS UP
pub const PAGE_NUMBER_2:                         usize = 0o_____________________100_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 2 LEVELS UP
pub const PAGE_NUMBER_3:                         usize = 0o_________________100_000_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 3 LEVELS UP
pub const PAGE_NUMBER_4:                         usize = 0o_____________100_000_000_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 4 LEVELS UP
pub const PAGE_NUMBER_5:                         usize = 0o_________100_000_000_000_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 5 LEVELS UP
pub const KIB:                                   usize = 0o_________________________2000_usize;          //ONE KIBIBYTE
pub const MIB:                                   usize = 0o_____________________400_0000_usize;          //ONE MEBIBYTE
pub const GIB:                                   usize = 0o_______________1_000_000_0000_usize;          //ONE GIBIBYTE
pub const TIB:                                   usize = 0o___________2_000_000_000_0000_usize;          //ONE TEBIBYTE
pub const PIB:                                   usize = 0o_______4_000_000_000_000_0000_usize;          //ONE PEBIBYTE


// MACROS
//Numeric Enum
macro_rules!numeric_enum {(
        #[repr($repr:ident)]
        $(#[$a:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $value:expr,)*
        }
    ) => {
        #[repr($repr)]
        $(#[$a])*
        $vis enum $name {
            $($variant = $value,)*
        }
        impl TryFrom<$repr> for $name {
            type Error = ();
            fn try_from(from: $repr) -> Result<Self, ()> {
                match from {
                    $($value => Ok(Self::$variant),)*
                    _ => Err(())
                }
            }
        }
    }
}


// TRAITS
//Locational Read
pub trait LocationalRead {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str>;
}


// STRUCTS
//Full ELF File Header
#[derive(Debug)]
pub struct ELFFile<'a, LR: 'a+LocationalRead> {
    pub file: &'a LR,
    pub header:   Header,
}
impl<'a, LR: 'a+LocationalRead> ELFFile<'a, LR> {
    // CONSTRUCTOR
    pub fn new(file: &'a LR) -> Result<ELFFile<'a, LR>, &'static str> {
        //Load File Header
        let file_header = Header::new(&{
            let mut buffer:[u8; 0x40] = [0u8; 0x40]; 
            file.read(0x00, &mut buffer)?;
            buffer}
        )?;
        //Return
        return Ok(ELFFile {
            file: file,
            header: file_header,
        });
    }

    // ITERATORS
    pub fn programs(&self) -> ProgramIterator<LR> {
        ProgramIterator::new(self.file, &self.header)
    }

    pub fn sections(&self) -> SectionIterator<LR> {
        SectionIterator::new(self.file, &self.header)
    }

    // FUNCTIONS
    //Total memory size of program from lowest virtual address to highest virtual address
    pub fn program_memory_size(&mut self) -> u64 {
        //Buffers
        let mut program_lowest_address:  u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let mut program_highest_address: u64 = 0x0000_0000_0000_0000;
        let mut loadable_found: bool = false;
        //Loop over program headers
        for program in ProgramIterator::new(self.file, &self.header) {
            match program {
                Ok(program) => {
                    //Check if program segment is loadable
                    if program.program_type == program::ProgramType::Loadable {
                        loadable_found = true;
                        //Check if minimum virtual address needs adjusting
                        if program.virtual_address < program_lowest_address {
                            program_lowest_address = program.virtual_address;
                        }
                        //Check if maximum virtual address needs adjusting
                        if program.virtual_address + program.memory_size > program_highest_address {
                            program_highest_address = program.virtual_address + program.memory_size;
                        }
                    }
                }
                Err(_) => {},
            }
        }
        //Return
        return if loadable_found {program_highest_address - program_lowest_address} else {0};
    }

    //Load File Into Memory (Very Important to Allocate Memory First)
    pub fn load(&mut self, location: *mut u8) -> Result<(), &'static str> {
        for program in ProgramIterator::new(self.file, &self.header) {
            match program {
                Ok(program) => {
                    if program.program_type == ProgramType::Loadable {
                        const BUFFER_SIZE: usize = 512;
                        let mut buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];
                        let count = program.file_size as usize/BUFFER_SIZE;
                        for i in 0..count {
                            self.file.read(program.file_offset as usize+i*BUFFER_SIZE, &mut buffer)?;
                            unsafe {copy_nonoverlapping(buffer.as_ptr(), location.add(program.virtual_address as usize + i*BUFFER_SIZE as usize), BUFFER_SIZE)}
                        }
                        let leftover: usize = program.file_size as usize %BUFFER_SIZE;
                        if leftover != 0 {
                            self.file.read(program.file_offset as usize + count*BUFFER_SIZE, &mut buffer[0..leftover])?;
                            unsafe {copy_nonoverlapping(buffer.as_ptr(), location.add(program.virtual_address as usize + count*BUFFER_SIZE as usize), leftover)}
                        }
                    }
                }
                Err(_) => {},
            }
        }
        return Ok(());
    }

    //Do Relocation (Very Important to Load First) **NOT FINISHED**
    pub fn relocate(&mut self, location: *mut u8) -> Result<(), &'static str> {
        if self.header.object_type != ObjectType::Shared {return Err("ELF File: Object Type Not Yet Supported for Relocation.")}
        for program in ProgramIterator::new(self.file, &self.header) {
            match program {
                Ok(program) => {
                    if program.program_type == ProgramType::Dynamic {
                        let mut symbol_table_address:                 Option<*const u8> = None;
                        let mut symbol_table_entry_size:              Option<u64>       = None;
                        let mut string_table_address:                 Option<*const u8> = None;
                        let mut string_table_size:                    Option<u64>       = None;
                        let mut implicit_relocation_table_address:    Option<*const u8> = None;
                        let mut implicit_relocation_table_size:       Option<u64>       = None;
                        let mut implicit_relocation_table_entry_size: Option<u64>       = None;
                        let mut explicit_relocation_table_address:    Option<*const u8> = None;
                        let mut explicit_relocation_table_size:       Option<u64>       = None;
                        let mut explicit_relocation_table_entry_size: Option<u64>       = None;
                        let mut procedure_linkage_table_address:      Option<*const u8> = None;
                        let mut procedure_linkage_table_size:         Option<u64>       = None;
                        let mut procedure_linkage_table_type:         RelocationType    = RelocationType::Explicit;
                    
                        for dynamic_entry_read in ProgramDynamicEntryIterator::new(self.file, &self.header, &program) {
                            match dynamic_entry_read {
                                Ok(dynamic_entry) => {
                                    match dynamic_entry.entry_type {
                                        ProgramDynamicEntryType::Null                             => return Err("ELF File: Encountered Null Dynamic Entry During Relocation."),
                                        ProgramDynamicEntryType::ProcedureLinkageTableSize        => procedure_linkage_table_size         = Some(dynamic_entry.value),
                                        ProgramDynamicEntryType::ProcedureLinkageTableAddress     => procedure_linkage_table_address      = Some(dynamic_entry.value as *mut u8),
                                        ProgramDynamicEntryType::StringTableAddress               => string_table_address                 = Some(dynamic_entry.value as *mut u8),
                                        ProgramDynamicEntryType::SymbolTableAddress               => symbol_table_address                 = Some(dynamic_entry.value as *mut u8),
                                        ProgramDynamicEntryType::ExplicitRelocationTableAddress   => explicit_relocation_table_address    = Some(dynamic_entry.value as *mut u8),
                                        ProgramDynamicEntryType::ExplicitRelocationTableSize      => explicit_relocation_table_size       = Some(dynamic_entry.value),
                                        ProgramDynamicEntryType::ExplicitRelocationTableEntrySize => explicit_relocation_table_entry_size = Some(dynamic_entry.value),
                                        ProgramDynamicEntryType::StringTableSize                  => string_table_size                    = Some(dynamic_entry.value),
                                        ProgramDynamicEntryType::SymbolTableEntrySize             => symbol_table_entry_size              = Some(dynamic_entry.value),
                                        ProgramDynamicEntryType::ImplicitRelocationTableAddress   => implicit_relocation_table_address    = Some(dynamic_entry.value as *mut u8),
                                        ProgramDynamicEntryType::ImplicitRelocationTableSize      => implicit_relocation_table_size       = Some(dynamic_entry.value),
                                        ProgramDynamicEntryType::ImplicitRelocationTableEntrySize => implicit_relocation_table_entry_size = Some(dynamic_entry.value),
                                        ProgramDynamicEntryType::ProcedureLinkageTableType        => procedure_linkage_table_type         = RelocationType::try_from(dynamic_entry.value).map_err(|_| "ELF File: Encountered Invalid PLT Type Value.")?,
                                        _ => {},
                                    }
                                }
                                Err(_) => (),
                            }
                        }
                        return Err("ELF File: Relocation not yet implemented.")
                    }
                },
                Err(_) => {},
            }
        }
        return Err("ELF File: Dynamic Relocation Program Header Not Found.");
    }
}


// MODULES
//ELF File Header
pub mod header {
    // IMPORTS
    use core::convert::{TryFrom, TryInto};
    use crate::{BitWidth, Endianness};

    // STRUCTS
    //File Header
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub struct Header {
        pub bit_width:                 BitWidth,
        pub endianness:                Endianness,
        pub ident_version:             IdentVersion,
        pub binary_interface:          ApplicationBinaryInterface,
        pub binary_interface_version:  u8,
        pub object_type:               ObjectType,
        pub architecture:              InstructionSetArchitecture,
        pub version:                   Version,
        pub entry_point:               u64,
        pub program_header_offset:     u64,
        pub section_header_offset:     u64,
        pub flags:                     [u8;4],
        pub header_size:               u16,
        pub program_header_entry_size: u16,
        pub program_header_number:     u16,
        pub section_header_entry_size: u16,
        pub section_header_number:     u16,
        pub string_section_index:      u16,
    }
    impl Header {
        // CONSTRUCTOR
        pub fn new(bytes: &[u8]) -> Result<Header, &'static str> {
            if bytes.len()       <  0x10                             {return Err("ELF File Header: Length of data given to parse from not large enough to contain ident.")}
            if bytes[0x00..0x04] != [0x7Fu8, 0x45u8, 0x4cu8, 0x46u8] {return Err("ELF File Header: Invalid Magic Number (ei_magic).");}
            if bytes[0x04]       != 0x02                             {return Err("ELF File Header: Handling of Bit Width (ei_class) value of 32 bits (0x01) not yet handled.")}
            if bytes.len()       <  0x40                             {return Err("ELF File Header: Length of data given to parse from not large enough to contain header.")}
            if bytes[0x09..0x10] != [0x00u8; 7]                      {return Err("ELF File Header: Invalid Padding (ei_pad).")}
            let endianness:Endianness = Endianness::try_from(bytes[0x05]).map_err(|_| "ELF File Header: Invalid Endianness (ei_data).")?;
            let (u16_fb, u32_fb, u64_fb): (fn([u8;2])->u16, fn([u8;4])->u32, fn([u8;8])->u64) = match endianness{
                Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
                Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
            };
            return Result::Ok(Header {
                bit_width:                                   BitWidth::try_from(       bytes[0x04]                           ).map_err(|_| "ELF File Header: Invalid Bit Width (ei_class).")?,
                endianness,
                ident_version:                           IdentVersion::try_from(       bytes[0x06]                           ).map_err(|_| "ELF File Header: Invalid Ident Version (ei_version).")?,
                binary_interface:          ApplicationBinaryInterface::try_from(       bytes[0x07]                           ).map_err(|_| "ELF File Header: Invalid Application Binary Interface (ei_osabi).")?,
                binary_interface_version:                                              bytes[0x08],
                object_type:                               ObjectType::try_from(u16_fb(bytes[0x10..0x12].try_into().unwrap())).map_err(|_| "ELF File Header: Invalid Object Type (e_type).")?,
                architecture:              InstructionSetArchitecture::try_from(u16_fb(bytes[0x12..0x14].try_into().unwrap())).map_err(|_| "ELF File Header: Invalid Instruction Set Architecture (e_machine).")?,
                version:                                      Version::try_from(u32_fb(bytes[0x14..0x18].try_into().unwrap())).map_err(|_| "ELF File Header: Invalid ELF Version (e_version).")?,
                entry_point:                                                    u64_fb(bytes[0x18..0x20].try_into().unwrap()),
                program_header_offset:                                    match u64_fb(bytes[0x20..0x28].try_into().unwrap()) {0x40 => 0x40, _ => return Err("ELF File Header: Invalid Program Header Offset (e_phoff).")},
                section_header_offset:                                          u64_fb(bytes[0x28..0x30].try_into().unwrap()),
                flags:                                                                 bytes[0x30..0x34].try_into().unwrap(),
                header_size:                                              match u16_fb(bytes[0x34..0x36].try_into().unwrap()) {0x40 => 0x40, _ => return Err("ELF File Header: Invalid ELF Header Size (e_ehsize).")},
                program_header_entry_size:                                match u16_fb(bytes[0x36..0x38].try_into().unwrap()) {0x38 => 0x38, _ => return Err("ELF File Header: Invalid Program Header Entry Size (e_phentsize).")},
                program_header_number:                                          u16_fb(bytes[0x38..0x3A].try_into().unwrap()),
                section_header_entry_size:                                match u16_fb(bytes[0x3A..0x3C].try_into().unwrap()) {0x40 => 0x40, _ => return Err("ELF File Header: Invalid Section Header Entry Size (e_shentsize).")},
                section_header_number:                                          u16_fb(bytes[0x3C..0x3E].try_into().unwrap()),
                string_section_index:                                     match u16_fb(bytes[0x3E..0x40].try_into().unwrap()) {a => if a < u16_fb(bytes[0x3C..0x3E].try_into().unwrap()){a} else {return Err("ELF File Header: Invalid String Section Index (e_shstrndx) according to Section Header Number (e_shnum).")}},
            })
        }
    }

    // ENUMS
    //ELF Ident Version
    numeric_enum! {
        #[repr(u8)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum IdentVersion {
            Original = 0x01,
        }
    }
    
    //Application Binary Interface
    numeric_enum! {
        #[repr(u8)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum ApplicationBinaryInterface {
            None              = 0x00,
            HewettPackardUnix = 0x01,
            NetBSD            = 0x02,
            Linux             = 0x03,
            GNUHurd           = 0x04,
            SOLARIS           = 0x06,
            AIX               = 0x07,
            IRIX              = 0x08,
            FreeBSD           = 0x09,
            Tru64Unix         = 0x0A,
            NovelleModesto    = 0x0B,
            OpenBSD           = 0x0C,
            OpenVMS           = 0x0D,
            Nonstop           = 0x0E,
            AROS              = 0x0F,
            Fenix             = 0x10,
            Cloud             = 0x11,
            VOS               = 0x12,
        }
    }
    
    //Object Type
    numeric_enum! {
        #[repr(u16)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum ObjectType {
            None        = 0x00,
            Relocatable = 0x01,
            Executable  = 0x02,
            Shared      = 0x03,
            Core        = 0x04,
        }
    }
    
    //Instruction Set Architecture
    numeric_enum! {
        #[repr(u16)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum InstructionSetArchitecture {
            None          = 0x0000,
            EmM32         = 0x0001,
            EmSparc       = 0x0002,
            Em386         = 0x0003,
            Em68K         = 0x0004,
            Em88K         = 0x0005,
            Em860         = 0x0007,
            EmMips        = 0x0008,
            EmS370        = 0x0009,
            EmMipsRS3LE   = 0x000A,
            EmPaRisc      = 0x000F,
            EmVPP500      = 0x0011,
            EmSparc32Plus = 0x0012,
            Em960         = 0x0013,
            EmPPC         = 0x0014,
            EmPPC64       = 0x0015,
            EmS390        = 0x0016,
            EmV800        = 0x0024,
            EmFR20        = 0x0025,
            EmRH32        = 0x0026,
            EmRCE         = 0x0027,
            EmARM         = 0x0028,
            EmAlpha       = 0x0029,
            EmSH          = 0x002A,
            EmSparcV9     = 0x002B,
            EmTriCore     = 0x002C,
            EmARC         = 0x002D,
            EmH8300       = 0x002E,
            EmH8300H      = 0x002F,
            EmH8S         = 0x0030,
            EmH8500       = 0x0031,
            EmIA64        = 0x0032,
            EmMipsX       = 0x0033,
            EmColdFire    = 0x0034,
            Em68HC12      = 0x0035,
            EmMMA         = 0x0036,
            EmPCP         = 0x0037,
            EmNCPU        = 0x0038,
            EmNDR1        = 0x0039,
            EmStarCore    = 0x003A,
            EmME16        = 0x003B,
            EmST100       = 0x003C,
            EmTinyJ       = 0x003D,
            EmX86_64      = 0x003E,
            EmPDSP        = 0x003F,
            EmPDP10       = 0x0040,
            EmPDP11       = 0x0041,
            EmFX66        = 0x0042,
            EmST9Plus     = 0x0043,
            EmST7         = 0x0044,
            Em68HC16      = 0x0045,
            Em68HC11      = 0x0046,
            Em68HC08      = 0x0047,
            Em68HC05      = 0x0048,
            EmSVx         = 0x0049,
            EmST19        = 0x004A,
            EmVAX         = 0x004B,
            EmCRIS        = 0x004C,
            EmJavelin     = 0x004D,
            EmFirepath    = 0x004E,
            EmZSP         = 0x004F,
            EmMMIX        = 0x0050,
            EmHUANY       = 0x0051,
            EmPrism       = 0x0052,
            EmAVR         = 0x0053,
            EmFR30        = 0x0054,
            EmD10V        = 0x0055,
            EmD30V        = 0x0056,
            EmV850        = 0x0057,
            EmM32R        = 0x0058,
            EmMN10300     = 0x0059,
            EmMN10200     = 0x005A,
            EmPJ          = 0x005B,
            EmOpenRISC    = 0x005C,
            EmARCA5       = 0x005D,
            EmXtensa      = 0x005E,
            EmVideoCore   = 0x005F,
            EmTMMGPP      = 0x0060,
            EmNS32K       = 0x0061,
            EmTPC         = 0x0062,
            EmPNP1K       = 0x0063,
            EmST200       = 0x0064,
        }
    }

    //ELF Version
    numeric_enum! {
        #[repr(u32)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum Version {
            Original = 0x01,
        }
    }
}

//Program
pub mod program {
    // IMPORTS
    use core::convert::{TryFrom, TryInto};
    use crate::{BitWidth, Endianness, LocationalRead, header::Header};

    // STRUCTS
    //Program Header Iterator
    pub struct ProgramIterator<'a, LR: 'a+LocationalRead> {
        file:       &'a LR,
        bit_width:      BitWidth,
        endianness:     Endianness,
        base_offset:    u64,
        entry_position: usize,
        entry_count:    usize,
    }
    impl<'a, LR: 'a+LocationalRead> ProgramIterator<'a, LR> {
        // FUNCTIONS
        //Constructor
        pub fn new(file: &'a LR, file_header: &Header) -> Self{
            Self {
                file:           file,
                bit_width:      file_header.bit_width,
                endianness:     file_header.endianness,
                base_offset:    file_header.program_header_offset,
                entry_position: 0,
                entry_count:    file_header.program_header_number as usize,
            }
        }
        
        //Get Entry
        fn entry(&mut self) -> Result<Program, &'static str> {
            return match self.bit_width {
                BitWidth::W32 => {
                    let mut buffer: [u8; 0x20] = [0u8; 0x20];
                    self.file.read(self.base_offset as usize + 0x20*self.entry_position as usize, &mut buffer)?;
                    Program::new(&buffer, self.bit_width, self.endianness)
                },
                BitWidth::W64 => {
                    let mut buffer: [u8; 0x38] = [0u8; 0x38];
                    self.file.read(self.base_offset as usize + 0x38*self.entry_position as usize, &mut buffer)?;
                    Program::new(&buffer, self.bit_width, self.endianness)
                },
            }
        }
    }
    impl<'a, LR: 'a+LocationalRead> Iterator for ProgramIterator<'a, LR> {
        type Item = Result<Program, &'static str>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.entry_position >= self.entry_count {
                None
            }
            else {
                let entry = self.entry();
                self.entry_position += 1;
                Some(entry)
            }
        }
    }

    //Program Header
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub struct Program {
        pub program_type:        ProgramType,
        pub flags:               u32,
        pub file_offset:         u64,
        pub virtual_address:     u64,
        pub physical_address:    u64,
        pub file_size:           u64,
        pub memory_size:         u64,
        pub alignment:           u64,
    }
    impl Program {
        // CONSTRUCTOR
        //New
        pub fn new(data: &[u8], bit_width: BitWidth, endianness: Endianness) -> Result<Self, &'static str> {
            
            let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
                Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
                Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
            };
            return match bit_width {
                BitWidth::W32 => {
                    if data.len() != 0x20 {return Err("Program: Incorrect data length for 32-bit program.")}
                    Ok(Self {
                        program_type:     ProgramType::try_from(
                                          u32_fb(data[0x00..0x04].try_into().map_err( |_| "ELF Program Header: Error slicing program type."    )?))
                                                                            .map_err( |_| "ELF Program Header: Invalid program type."          )?,
                        file_offset:      u64_fb(data[0x04..0x08].try_into().map_err( |_| "ELF Program Header: Error slicing file offset."     )?),
                        virtual_address:  u64_fb(data[0x08..0x0C].try_into().map_err( |_| "ELF Program Header: Error slicing virtual address." )?),
                        physical_address: u64_fb(data[0x0C..0x10].try_into().map_err( |_| "ELF Program Header: Error slicing physical address.")?),
                        file_size:        u64_fb(data[0x20..0x28].try_into().map_err( |_| "ELF Program Header: Error slicing file size."       )?),
                        memory_size:      u64_fb(data[0x28..0x30].try_into().map_err( |_| "ELF Program Header: Error slicing memory size."     )?),
                        flags:            u32_fb(data[0x04..0x08].try_into().map_err( |_| "ELF Program Header: Error slicing flags."           )?),
                        alignment:        u64_fb(data[0x30..0x38].try_into().map_err( |_| "ELF Program Header: Error slicing alignment."       )?),
                    })
                },
                BitWidth::W64 => {
                    if data.len() != 0x38 {return Err("Program: Incorrect data length for 64-bit program.")};
                    Ok(Self {
                        program_type:     ProgramType::try_from(
                                          u32_fb(data[0x00..0x04].try_into().map_err( |_| "ELF Program Header: Error slicing program type."    )?))
                                                                            .map_err( |_| "ELF Program Header: Invalid program type."          )?,
                        flags:            u32_fb(data[0x04..0x08].try_into().map_err( |_| "ELF Program Header: Error slicing flags."           )?),
                        file_offset:      u64_fb(data[0x08..0x10].try_into().map_err( |_| "ELF Program Header: Error slicing file offset."     )?),
                        virtual_address:  u64_fb(data[0x10..0x18].try_into().map_err( |_| "ELF Program Header: Error slicing virtual address." )?),
                        physical_address: u64_fb(data[0x18..0x20].try_into().map_err( |_| "ELF Program Header: Error slicing physical address.")?),
                        file_size:        u64_fb(data[0x20..0x28].try_into().map_err( |_| "ELF Program Header: Error slicing file size."       )?),
                        memory_size:      u64_fb(data[0x28..0x30].try_into().map_err( |_| "ELF Program Header: Error slicing memory size."     )?),
                        alignment:        u64_fb(data[0x30..0x38].try_into().map_err( |_| "ELF Program Header: Error slicing alignment."       )?),
                    })
                }, 
            }
        }
    }

    // ENUMS
    //Program Type
    numeric_enum! {
        #[repr(u32)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum ProgramType {
            Null                 = 0x00_00_00_00,
            Loadable             = 0x00_00_00_01,
            Dynamic              = 0x00_00_00_02,
            Interpreter          = 0x00_00_00_03,
            Note                 = 0x00_00_00_04,
            ProgramHeader        = 0x00_00_00_06,
            ThreadLocalStorage   = 0x00_00_00_07,
        }
    }
}

//Section
pub mod section {
    // IMPORTS
    use core::convert::{TryFrom, TryInto};
    use crate::{BitWidth, Endianness, LocationalRead, header::Header};

    // STRUCTS
    //Section Header Iterator
    pub struct SectionIterator<'a, LR: 'a+LocationalRead> {
        file:   &'a     LR,
        bit_width:      BitWidth,
        endianness:     Endianness,
        base_offset:    u64,
        entry_position: usize,
        entry_count:    usize,
    }
    impl<'a, LR: 'a+LocationalRead> SectionIterator<'a, LR> {
        // FUNCTIONS
        //Constructor
        pub fn new(file: &'a LR, file_header: &Header) -> Self{
            Self {
                file:           file,
                bit_width:      file_header.bit_width,
                endianness:     file_header.endianness,
                base_offset:    file_header.section_header_offset,
                entry_position: 0,
                entry_count:    file_header.section_header_number as usize,
            }
        }
        
        //Get Entry
        fn entry(&mut self) -> Result<Section, &'static str> {
            return match self.bit_width {
                BitWidth::W32 => {
                    let mut buffer: [u8; 0x28] = [0u8; 0x28];
                    self.file.read(self.base_offset as usize + 0x28*self.entry_position as usize, &mut buffer)?;
                    Section::new(&buffer, self.bit_width, self.endianness)
                },
                BitWidth::W64 => {
                    let mut buffer: [u8; 0x40] = [0u8; 0x40];
                    self.file.read(self.base_offset as usize + 0x40*self.entry_position as usize, &mut buffer)?;
                    Section::new(&buffer, self.bit_width, self.endianness)
                },
            }
        }
    }
    impl<'a, LR: 'a+LocationalRead> Iterator for SectionIterator<'a, LR> {
        type Item = Result<Section, &'static str>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.entry_position >= self.entry_count {
                None
            }
            else {
                let entry = self.entry();
                self.entry_position += 1;
                Some(entry)
            }
        }
    }

    //Section Header
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub struct Section {
        pub name:            u32,
        pub section_type:    SectionType,
        pub flags:           u64,
        pub virtual_address: u64,
        pub file_offset:     u64,
        pub file_size:       u64,
        pub link:            u32,
        pub info:            u32,
        pub alignment:       u64,
        pub entry_size:      u64,
    }
    impl Section {
        // CONSTRUCTOR
        //New
        pub fn new(data: &[u8], bit_width: BitWidth, endianness: Endianness) -> Result<Self, &'static str> {
            let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
                Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
                Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
            };
            return match bit_width {
                BitWidth::W32 => {
                    if data.len() != 0x28 {return Err("Section: Incorrect data length for 32-bit section.")};
                    Ok(Self {
                        name:            u32_fb(data[0x00..0x04].try_into().map_err( |_| "Section: Error slicing name."           )?),
                        section_type: SectionType::try_from(
                                         u32_fb(data[0x04..0x08].try_into().map_err( |_| "Section: Error slicing section type."   )?))
                                                                           .map_err( |_| "Section: Invalid section type."         )?,
                        flags:           u32_fb(data[0x08..0x0C].try_into().map_err( |_| "Section: Error slicing flags."          )?) as u64,
                        virtual_address: u32_fb(data[0x0C..0x10].try_into().map_err( |_| "Section: Error slicing virtual address.")?) as u64,
                        file_offset:     u32_fb(data[0x10..0x14].try_into().map_err( |_| "Section: Error slicing file offset."    )?) as u64,
                        file_size:       u32_fb(data[0x14..0x18].try_into().map_err( |_| "Section: Error slicing file size."      )?) as u64,
                        link:            u32_fb(data[0x18..0x1C].try_into().map_err( |_| "Section: Error slicing link."           )?),
                        info:            u32_fb(data[0x1C..0x20].try_into().map_err( |_| "Section: Error slicing info."           )?),
                        alignment:       u32_fb(data[0x20..0x24].try_into().map_err( |_| "Section: Error slicing alignment."      )?) as u64,
                        entry_size:      u32_fb(data[0x24..0x28].try_into().map_err( |_| "Section: Error slicing entry size."     )?) as u64,
                    })
                },
                BitWidth::W64 => {
                    if data.len() != 0x40 {return Err("Section: Incorrect data length for 64-bit section.")};
                    Ok(Self {
                        name:            u32_fb(data[0x00..0x04].try_into().map_err( |_| "Section: Error slicing name."           )?),
                        section_type: SectionType::try_from(
                                         u32_fb(data[0x04..0x08].try_into().map_err( |_| "Section: Error slicing section type."   )?))
                                                                           .map_err( |_| "Section: Invalid section type."         )?,
                        flags:           u64_fb(data[0x08..0x10].try_into().map_err( |_| "Section: Error slicing flags."          )?),
                        virtual_address: u64_fb(data[0x10..0x18].try_into().map_err( |_| "Section: Error slicing virtual address.")?),
                        file_offset:     u64_fb(data[0x18..0x20].try_into().map_err( |_| "Section: Error slicing file offset."    )?),
                        file_size:       u64_fb(data[0x20..0x28].try_into().map_err( |_| "Section: Error slicing file size."      )?),
                        link:            u32_fb(data[0x28..0x2C].try_into().map_err( |_| "Section: Error slicing link."           )?),
                        info:            u32_fb(data[0x2C..0x30].try_into().map_err( |_| "Section: Error slicing info."           )?),
                        alignment:       u64_fb(data[0x30..0x38].try_into().map_err( |_| "Section: Error slicing alignment."      )?),
                        entry_size:      u64_fb(data[0x38..0x40].try_into().map_err( |_| "Section: Error slicing entry size."     )?),
                    })
                },
            }
        }
    }

    // ENUMS
    //Section Type
    numeric_enum!{
        #[repr(u32)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum SectionType {
            Null                           = 0x00_00_00_00,
            ProgramData                    = 0x00_00_00_01,
            SymbolTable                    = 0x00_00_00_02,
            StringTable                    = 0x00_00_00_03,
            ExplicitRelocationTable        = 0x00_00_00_04,
            SymbolHashTable                = 0x00_00_00_05,
            DynamicLinkingInformation      = 0x00_00_00_06,
            Notes                          = 0x00_00_00_07,
            ProgramNoData                  = 0x00_00_00_08,
            ImplicitRelocationTable        = 0x00_00_00_09,
            DynamicSymbolTable             = 0x00_00_00_0B,
            InitializationFunctionArray    = 0x00_00_00_0E,
            TerminationFunctionArray       = 0x00_00_00_0F,
            PreInitializationFunctionArray = 0x00_00_00_10,
            SectionGroup                   = 0x00_00_00_11,
            ExtendedSectionIndex           = 0x00_00_00_12,
            Number                         = 0x00_00_00_13,
        }
    }
}

//Program: Dynamic Entry
pub mod program_dynamic_entry {
    // IMPORTS
    use core::convert::{TryFrom, TryInto};
    use crate::{BitWidth, Endianness, LocationalRead, header::Header, program::Program};
    
    // STRUCTS
    //Dynamic Table Iterator
    pub struct ProgramDynamicEntryIterator<'a, LR: 'a+LocationalRead> {
        file:       &'a LR,
        bit_width:      BitWidth,
        endianness:     Endianness,
        base_offset:    u64,
        entry_position: u64,
        entry_count:    u64,
    }
    impl<'a, LR: 'a+LocationalRead> ProgramDynamicEntryIterator<'a, LR> {
        // FUNCTIONS
        //Constructor
        pub fn new(file: &'a LR, file_header: &Header, program_header: &Program) -> Self{
            ProgramDynamicEntryIterator {
                file:           file,
                bit_width:      file_header.bit_width,
                endianness:     file_header.endianness,
                base_offset:    program_header.file_offset,
                entry_position: 0,
                entry_count:    program_header.file_size / match file_header.bit_width{BitWidth::W32 => 8, BitWidth::W64 => 16},
            }
        }

        //Get Entry
        fn entry(&mut self) -> Result<ProgramDynamicEntry, &'static str> {
            return match self.bit_width {
                BitWidth::W32 => {
                    let mut buffer: [u8; 8] = [0u8; 8];
                    self.file.read(self.base_offset as usize + 8*self.entry_position as usize, &mut buffer)?;
                    ProgramDynamicEntry::new(&buffer, self.bit_width, self.endianness)
                },
                BitWidth::W64 => {
                    let mut buffer: [u8; 16] = [0u8; 16];
                    self.file.read(self.base_offset as usize + 16*self.entry_position as usize, &mut buffer)?;
                    ProgramDynamicEntry::new(&buffer, self.bit_width, self.endianness)
                },
            }
        }
    }
    impl<'a, R: 'a+LocationalRead> Iterator for ProgramDynamicEntryIterator<'a, R> {
        type Item = Result<ProgramDynamicEntry, &'static str>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.entry_position >= self.entry_count {
                None
            }
            else {
                let entry = self.entry();
                match entry {
                    Ok(e) => if e.entry_type == ProgramDynamicEntryType::Null {return None}
                    Err(_) => (),
                }
                self.entry_position += 1;
                Some(entry)
            }
        }
    }
    
    //Dynamic Entry
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub struct ProgramDynamicEntry {
        pub entry_type: ProgramDynamicEntryType,
        pub value:      u64,
    }
    impl ProgramDynamicEntry {
        pub fn new(data: &[u8], bit_width: BitWidth, endianness: Endianness) -> Result<Self, &'static str> {
            let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
                Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
                Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
            };
            match bit_width {
                BitWidth::W32 => {
                    if data.len() != 8 {return Err("Program Dynamic: Incorrect data length for 32-bit program dynamic entry.")};
                    Ok(Self {
                        entry_type: ProgramDynamicEntryType::try_from(
                               u32_fb(data[0x00..0x04].try_into().map_err( |_| "Dynamic Entry: Error slicing entry type.")?) as u64)
                                                           .map_err( |_| "Dynamic Entry: Invalid entry type."      )?,
                        value: u32_fb(data[0x04..0x08].try_into().map_err( |_| "Dynamic Entry: Error slicing value."     )?) as u64,
                    })
                },
                BitWidth::W64 => {
                    if data.len() != 16 {return Err("Dynamic Entry: Incorrect data length.")};
                    Ok(Self {
                        entry_type: ProgramDynamicEntryType::try_from(
                               u64_fb(data[0x00..0x08].try_into().map_err( |_| "Dynamic Entry: Error slicing entry type.")?) as u64)
                                                            .map_err( |_| "Dynamic Entry: Invalid entry type."      )?,
                        value: u64_fb(data[0x08..0x10].try_into().map_err( |_| "Dynamic Entry: Error slicing value."     )?) as u64,
                    })
                },
            }
        }
    }

    // ENUMS
    //Dynamic Entry Type
    numeric_enum! {
        #[repr(u64)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum ProgramDynamicEntryType {
            Null                                     = 0x00,
            Needed                                   = 0x01,
            ProcedureLinkageTableSize                = 0x02,
            ProcedureLinkageTableAddress             = 0x03,
            SymbolHashTableAddress                   = 0x04,
            StringTableAddress                       = 0x05,
            SymbolTableAddress                       = 0x06,
            ExplicitRelocationTableAddress           = 0x07,
            ExplicitRelocationTableSize              = 0x08,
            ExplicitRelocationTableEntrySize         = 0x09,
            StringTableSize                          = 0x0A,
            SymbolTableEntrySize                     = 0x0B,
            InitializationFunctionAddress            = 0x0C,
            TerminationFunctionAddress               = 0x0D,
            StringTableNameOffset                    = 0x0E,
            StringTablePathOffset                    = 0x0F,
            SymbolicResolutionFlag                   = 0x10,
            ImplicitRelocationTableAddress           = 0x11,
            ImplicitRelocationTableSize              = 0x12,
            ImplicitRelocationTableEntrySize         = 0x13,
            ProcedureLinkageTableType                = 0x14,
            DebugAddress                             = 0x15,
            TextRelocationFlag                       = 0x16,
            JumpRelocationAddress                    = 0x17,
            BindNowFlag                              = 0x18,
        }
    }
}

//Relocation Table
pub mod elf_relocation_table {
    // IMPORTS
    use crate::{BitWidth, Endianness, LocationalRead, RelocationType};

    // STRUCTS
    //Relocation Table Iterator
    struct RelocationTableIterator <'a, LR: 'a+LocationalRead> {
        file:  &'a mut   LR,
        bit_width:       BitWidth,
        endianness:      Endianness,
        relocation_type: RelocationType,
        file_offset:     u64,
        entry_positon:   usize,
        entry_count:     usize,
    }
    impl<'a, LR: 'a+LocationalRead> RelocationTableIterator<'a, LR> {
        // CONSTRUCTOR
        /*pub fn new(file: &'a mut LR, file_header: ELFFileHeader, file_size: usize) -> Self {
            RelocationTableIterator {
                file,
                bit_width: file_header.bit_width,
                endianness: file_header.endianness,
            }
        }*/
    }

}


// ENUMS
//Bit Width
numeric_enum! {
    #[repr(u8)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum BitWidth {
        W32 = 0x01,
        W64 = 0x02,
    }
}

//Endianness
numeric_enum! {
    #[repr(u8)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum Endianness {
        Little = 0x01,
        Big    = 0x02,
    }
}

//Relocation Type
numeric_enum!{
    #[repr(u64)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum RelocationType {
        Explicit = 0x07,
        Implicit = 0x11,
    }
}
