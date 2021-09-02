// GLUON
// Gluon is the Noble loading library:
// Memory locations of important objects
// Sizes and counts related to page tables
// Structs and Enums related to the contents and handling of ELF files
// Structs and Enums related to the contents and handling of x86-64 page tables


// HEADER
//Flags
#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::inconsistent_digit_grouping)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

//Imports
use core::convert::TryFrom;
use core::intrinsics::copy_nonoverlapping;
use header::*;
use program::*;
use section::*;
use relocation_entry::*;

//Constants
pub const GLUON_VERSION: &str = "vDEV-2021-08-24";                                                       //CURRENT VERSION OF GRAPHICS LIBRARY
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


// ELF FILES
//Locational Read
pub trait LocationalRead {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str>;
}

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
        Ok(ELFFile {
            file,
            header: file_header,
        })
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
        for program in ProgramIterator::new(self.file, &self.header).flatten() {
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
        //Return
        if loadable_found {program_highest_address - program_lowest_address} else {0}
    }

    //Load File Into Memory (Very Important to Allocate Memory First)
    pub unsafe fn load(&mut self, location: *mut u8) -> Result<(), &'static str> {
        let program_iterator = ProgramIterator::new(self.file, &self.header)
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .filter(|program| program.program_type == ProgramType::Loadable);
        for program in program_iterator {
            const BUFFER_SIZE: usize = 512;
            let mut buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];
            let count = program.file_size as usize/BUFFER_SIZE;
            for i in 0..count {
                self.file.read(program.file_offset as usize+i*BUFFER_SIZE, &mut buffer)?;
                copy_nonoverlapping(buffer.as_ptr(), location.add(program.virtual_address as usize + i*BUFFER_SIZE as usize), BUFFER_SIZE);
            }
            let leftover: usize = program.file_size as usize %BUFFER_SIZE;
            if leftover != 0 {
                self.file.read(program.file_offset as usize + count*BUFFER_SIZE, &mut buffer[0..leftover])?;
                copy_nonoverlapping(buffer.as_ptr(), location.add(program.virtual_address as usize + count*BUFFER_SIZE as usize), leftover);
            }
        }
        //Return
        Ok(())
    }

    //Do Relocation (Very Important to Load First) **NOT FINISHED**
    pub unsafe fn relocate(&mut self, loaded_location: *mut u8, reloc_location: *mut u8) -> Result<(), &'static str> {
        //Ensure correct object type
        if self.header.object_type != ObjectType::Shared {return Err("ELF File: Relocation not yet supported for this object type.")}
        //Find relocation entries
        let explicit_relocation_sections = self.sections()
                .filter(|result| result.is_ok())
                .map(|result| result.unwrap())
                .filter(|section| section.section_type == SectionType::ExplicitRelocationTable);
        //Match to file architecture
        match self.header.architecture {
            //x86-64 Relocation
            InstructionSetArchitecture::EmX86_64 => {
                for section in explicit_relocation_sections {
                    let relocation_table = RelocationEntryIterator::new(self.file, self.header.bit_width, self.header.endianness, RelocationType::Explicit, section.file_size, section.file_offset)
                    .filter(|result| result.is_ok())
                    .map(|result| result.unwrap());
                    for entry in relocation_table {
                        match entry.relocation_entry_type {
                            RelocationEntryTypeX86_64::R_X86_64_NONE      => {},
                            RelocationEntryTypeX86_64::R_X86_64_RELATIVE  => {*((loaded_location as u64 + entry.offset) as *mut u64) = (reloc_location as u64) + entry.addend.unwrap_or(0);},
                            _ => {return Err("ELF File: Relocation entry type found which is not supported.")},
                        }
                    }
                }
                //Return
                Ok(())
            },
            //Return Error
            _ => Err("ELF File: Relocation not supported for this file's Instruction Set Architecture.")
        }
    }
}

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
            Result::Ok(Header {
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
                string_section_index:                             {let a: u16 = u16_fb(bytes[0x3E..0x40].try_into().unwrap()); 
                                                                         if a < u16_fb(bytes[0x3C..0x3E].try_into().unwrap()) {a} else {return Err("ELF File Header: Invalid String Section Index (e_shstrndx) according to Section Header Number (e_shnum).")}},
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
                file,
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
            match bit_width {
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
                file,
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
            match bit_width {
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
                file,
                bit_width:      file_header.bit_width,
                endianness:     file_header.endianness,
                base_offset:    program_header.file_offset,
                entry_position: 0,
                entry_count:    program_header.file_size / match file_header.bit_width {BitWidth::W32 => 8, BitWidth::W64 => 16},
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
                if let Ok(e) = entry {if e.entry_type == ProgramDynamicEntryType::Null {return None}}
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
pub mod relocation_entry {
    // IMPORTS
    use core::convert::{TryFrom, TryInto};
    use crate::{BitWidth, Endianness, LocationalRead, RelocationType};

    // STRUCTS
    //Relocation Entry
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub struct RelocationEntry {
        pub offset:                u64,
        pub symbol:                u32,
        pub relocation_entry_type: RelocationEntryTypeX86_64,
        pub addend:                Option<u64>,
    }
    impl RelocationEntry {
        // CONSTRUCTOR
        pub fn new(data: &[u8], bit_width: BitWidth, endianness: Endianness, relocation_type: RelocationType) -> Result<Self, &'static str> {
            let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
                Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
                Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
            };
            match (bit_width, relocation_type) {
                (BitWidth::W32, RelocationType::Implicit) => {
                    if data.len() != 0x08 {return Err("Relocation Entry: Length of data provided incorrect for 32-bit entry with implicit addends.");}
                    let info: u32 = u32_fb(data[0x04..0x08].try_into().map_err( |_| "Relocation Entry: Error slicing info.")?);
                    Ok(Self {
                        offset: u32_fb(data[0x00..0x04].try_into().map_err( |_| "Relocation Entry: Error slicing offset.")?) as u64,
                        symbol: info >> 8,
                        relocation_entry_type: RelocationEntryTypeX86_64::try_from(info & 0xFF).map_err( |_| "Relocation Entry: Invalid relocation entry type.")?,
                        addend: None,
                    })
                },
                (BitWidth::W32, RelocationType::Explicit) => {
                    if data.len() != 0x0C {return Err("Relocation Entry: Length of data provided incorrect for 32-bit entry with explicit addends.");}
                    let info: u32 = u32_fb(data[0x04..0x08].try_into().map_err( |_| "Relocation Entry: Error slicing info.")?);
                    Ok(Self {
                        offset: u32_fb(data[0x00..0x04].try_into().map_err( |_| "Relocation Entry: Error slicing offset.")?) as u64,
                        symbol: info >> 8,
                        relocation_entry_type: RelocationEntryTypeX86_64::try_from(info & 0xFF).map_err( |_| "Relocation Entry: Invalid relocation entry type.")?,
                        addend: Some(u32_fb(data[0x08..0x0C].try_into().map_err( |_| "Relocation Entry: Error slicing addend.")?) as u64),
                    })
                },
                (BitWidth::W64, RelocationType::Implicit) => {
                    if data.len() != 0x10 {return Err("Relocation Entry: Length of data provided incorrect for 64-bit entry with implicit addends.");}
                    let info: u64 = u64_fb(data[0x08..0x10].try_into().map_err( |_| "Relocation Entry: Error slicing info.")?);
                    Ok(Self {
                        offset: u64_fb(data[0x00..0x08].try_into().map_err( |_| "Relocation Entry: Error slicing offset.")?),
                        symbol: (info>>32) as u32,
                        relocation_entry_type: RelocationEntryTypeX86_64::try_from((info & 0xFFFFFFFF) as u32).map_err( |_| "Relocation Entry: Invalid relocation Entry Type.")?,
                        addend: None,
                    })
                },
                (BitWidth::W64, RelocationType::Explicit) => {
                    if data.len() != 0x18 {return Err("Relocation Entry: Length of data provided incorrect for 64-bit entry with implicit addends.");}
                    let info: u64 = u64_fb(data[0x08..0x10].try_into().map_err( |_| "Relocation Entry: Error slicing info.")?);
                    Ok(Self {
                        offset: u64_fb(data[0x00..0x08].try_into().map_err( |_| "Relocation Entry: Error slicing offset.")?),
                        symbol: (info>>32) as u32,
                        relocation_entry_type: RelocationEntryTypeX86_64::try_from((info & 0xFFFFFFFF) as u32).map_err( |_| "Relocation Entry: Invalid relocation Entry Type.")?,
                        addend: Some(u64_fb(data[0x10..0x18].try_into().map_err( |_| "Relocation Entry: Error slicing info.")?)),
                    })
                },
            }
        }
    }

    //Relocation Entry Iterator
    pub struct RelocationEntryIterator <'a, LR: 'a+LocationalRead> {
        file:  &'a       LR,
        bit_width:       BitWidth,
        endianness:      Endianness,
        relocation_type: RelocationType,
        file_offset:     u64,
        entry_position:  u64,
        entry_count:     u64,
    }
    impl<'a, LR: 'a+LocationalRead> RelocationEntryIterator<'a, LR> {
        // CONSTRUCTOR
        pub fn new(file: &'a LR, bit_width: BitWidth, endianness: Endianness, relocation_type: RelocationType, file_size: u64, file_offset: u64) -> Self {
            Self {
                file,
                bit_width,
                endianness,
                relocation_type,
                file_offset,
                entry_position: 0,
                entry_count: file_size / match (bit_width, relocation_type) {
                    (BitWidth::W32, RelocationType::Implicit) => 0x08, (BitWidth::W32, RelocationType::Explicit) => 0x0C,
                    (BitWidth::W64, RelocationType::Implicit) => 0x10, (BitWidth::W64, RelocationType::Explicit) => 0x18,},
            }
        }

        // ENTRY
        pub fn entry(&mut self) -> Result<RelocationEntry, &'static str> {
            match (self.bit_width, self.relocation_type) {
                (BitWidth::W32, RelocationType::Implicit) => {
                    let mut buffer: [u8; 0x08] = [0u8; 0x08];
                    self.file.read(self.file_offset as usize + 0x08*self.entry_position as usize, &mut buffer)?;
                    RelocationEntry::new(&buffer, self.bit_width, self.endianness, self.relocation_type)
                },
                (BitWidth::W32, RelocationType::Explicit) => {
                    let mut buffer: [u8; 0x0C] = [0u8; 0x0C];
                    self.file.read(self.file_offset as usize + 0x0C*self.entry_position as usize, &mut buffer)?;
                    RelocationEntry::new(&buffer, self.bit_width, self.endianness, self.relocation_type)
                }, 
                (BitWidth::W64, RelocationType::Implicit) => {
                    let mut buffer: [u8; 0x10] = [0u8; 0x10];
                    self.file.read(self.file_offset as usize + 0x10*self.entry_position as usize, &mut buffer)?;
                    RelocationEntry::new(&buffer, self.bit_width, self.endianness, self.relocation_type)
                }, 
                (BitWidth::W64, RelocationType::Explicit) => {
                    let mut buffer: [u8; 0x18] = [0u8; 0x18];
                    self.file.read(self.file_offset as usize + 0x18*self.entry_position as usize, &mut buffer)?;
                    RelocationEntry::new(&buffer, self.bit_width, self.endianness, self.relocation_type)
                },
            }
        }
    }
    impl<'a, LR: 'a+LocationalRead> Iterator for RelocationEntryIterator<'a, LR> {
        type Item = Result<RelocationEntry, &'static str>;

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

    //ENUMS
    //Relocation Entry Type (x86-64)
    numeric_enum!{
        #[repr(u32)]
        #[derive(Clone, Copy)]
        #[derive(Debug)]
        pub enum RelocationEntryTypeX86_64 {
            R_X86_64_NONE            = 0x00,
            R_X86_64_64              = 0x01,
            R_X86_64_PC32            = 0x02,
            R_X86_64_GOT32           = 0x03,
            R_X86_64_PLT32           = 0x04,
            R_X86_64_COPY            = 0x05,
            R_X86_64_GLOB_DAT        = 0x06,
            R_X86_64_JUMP_SLOT       = 0x07,
            R_X86_64_RELATIVE        = 0x08,
            R_X86_64_GOTPCREL        = 0x09,
            R_X86_64_32              = 0x0A,
            R_X86_64_32S             = 0x0B,
            R_X86_64_16              = 0x0C,
            R_X86_64_PC16            = 0x0D,
            R_X86_64_8               = 0x0E,
            R_X86_64_PC8             = 0x0F,
            R_X86_64_DTPMOD64        = 0x10,
            R_X86_64_DTPOFF64        = 0x11,
            R_X86_64_TPOFF64         = 0x12,
            R_X86_64_TLSGD           = 0x13,
            R_X86_64_TLSLD           = 0x14,
            R_X86_64_DTPOFF32        = 0x15,
            R_X86_64_GOTTPOFF        = 0x16,
            R_X86_64_TPOFF32         = 0x17,
            R_X86_64_PC64            = 0x18,
            R_X86_64_GOTOFF64        = 0x19,
            R_X86_64_GOTPC32         = 0x1A,
            R_X86_64_SIZE32          = 0x20,
            R_X86_64_SIZE64          = 0x21,
            R_X86_64_GOTPC32_TLSDESC = 0x22,
            R_X86_64_TLSDESC_CALL    = 0x23,
            R_X86_64_TLSDESC         = 0x24,
            R_X86_64_IRELATIVE       = 0x25,
        }
    }
}

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


// PAGING
//Page Allocator
pub trait PageAllocator {
    fn allocate_page(&self) -> *mut u64;
}

//Page Table
pub struct PageMap {
    pub location:  *mut u64,
    map_level: PageMapLevel,
}
impl PageMap {
    //Constructor
    pub fn new(location: *mut u64, map_level: PageMapLevel) -> Result<Self, &'static str> {
        if location as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Map: Location not aligned to 4KiB boundary.")}
        Ok(Self {
            location,
            map_level,
        })
    }

    //Get an entry from a location
    pub fn read_entry(&self, position: usize) -> Result<PageMapEntry, &'static str> {
        if position >= PAGE_NUMBER_1 {return Err("Page Map: Entry index out of bounds during read.")}
        let data = unsafe{*(self.location.add(position))};
        PageMapEntry::from_u64(data, self.map_level)
    }

    //Write an entry to a location
    pub fn write_entry(&self, position: usize, entry: PageMapEntry) -> Result<(), &'static str> {
        if position >= PAGE_NUMBER_1 {return Err("Page Map: Entry index out of bounds during write.")}
        if entry.entry_level != self.map_level {return Err("Page Map: Entry level does not match map level.")}
        let data = entry.to_u64()?;
        unsafe {*(self.location.add(position)) = data}
        Ok(())
    }

    //Map pages from a physical and within-map offset
    pub fn  map_pages_offset(&self, page_allocator: &dyn PageAllocator, physical_offset: *mut u64, map_offset: usize, size: usize, write: bool, supervisor: bool, execute_disable: bool) -> Result<(), &'static str> {
        match self.map_level {
            PageMapLevel::L1 => {self.map_pages_offset_pml1(                physical_offset, map_offset, size, write, supervisor, execute_disable)},
            PageMapLevel::L2 => {self.map_pages_offset_pml2(page_allocator, physical_offset, map_offset, size, write, supervisor, execute_disable)},
            PageMapLevel::L3 => {self.map_pages_offset_pml3(page_allocator, physical_offset, map_offset, size, write, supervisor, execute_disable)},
            _ => Err("Page Map: Offset function not finished.")
        }
    }
    fn map_pages_offset_pml1(&self,                                     physical_offset: *mut u64, map_offset: usize, size: usize, write: bool, supervisor: bool, execute_disable: bool) -> Result<(), &'static str> {
        //TODO: Create macro to check and return if allocate_page doesn't create properly aligned page
        //Check Parameters
        if self.map_level                            != PageMapLevel::L1 {return Err("Page Map: Offset PML1 called on page map of wrong level.")}
        if physical_offset as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Physical offset not aligned to 4KiB boundary.")}
        if map_offset      as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Map offset not aligned to 4KiB boundary.")}
        if map_offset      +  size  > PAGE_SIZE_2MIB                     {return Err("Page Map: Map position and size requested does not fit within level 1 map boundaries.")}
        //Position variables
        let pages = size/PAGE_SIZE_4KIB + if size%PAGE_SIZE_4KIB != 0 {1} else {0};
        let position = map_offset / PAGE_SIZE_4KIB;
        //Loop
        for i in 0..pages {
            self.write_entry(i+position, PageMapEntry::new(PageMapLevel::L1, PageMapEntryType::Memory, unsafe {((physical_offset as *mut u8).add(i*PAGE_SIZE_4KIB)) as *mut u64}, write, supervisor, execute_disable)?)?;
        }
        //Return
        Ok(())
    }
    fn map_pages_offset_pml2(&self, page_allocator: &dyn PageAllocator, physical_offset: *mut u64, map_offset: usize, size: usize, write: bool, supervisor: bool, execute_disable: bool) -> Result<(), &'static str> {
        //Check Parameters
        if self.map_level                            != PageMapLevel::L2 {return Err("Page Map: Offset PML2 called on page map of wrong level.")}
        if physical_offset as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Physical offset not aligned to 4KiB boundary.")}
        if map_offset      as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Map offset not aligned to 4KiB boundary.")}
        if map_offset      +  size  > PAGE_SIZE_1GIB                     {return Err("Page Map: Map position and size requested does not fit within level 2 map boundaries.")}
        //Position Variables
        let start_position: usize =  map_offset         / PAGE_SIZE_2MIB;
        let start_offset:   usize =  map_offset         % PAGE_SIZE_2MIB;
        let end_size:       usize = (map_offset + size) % PAGE_SIZE_2MIB;
        let end_position:   usize = (map_offset + size) / PAGE_SIZE_2MIB + if end_size != 0 {1} else {0};
        //Loop
        for position in start_position..end_position {
            //Retrieve PML1
            let entry = match self.read_entry(position) {
                Ok(entry) => {
                    if entry.present {
                        entry
                    }
                    else {
                        let new_entry = PageMapEntry::new(PageMapLevel::L2, PageMapEntryType::Table, page_allocator.allocate_page(), write, supervisor, execute_disable)?;
                        self.write_entry(position, new_entry)?;
                        new_entry
                    }
                },
                Err(_) => {
                    let new_entry = PageMapEntry::new(PageMapLevel::L2, PageMapEntryType::Table, page_allocator.allocate_page(), write, supervisor, execute_disable)?;
                    self.write_entry(position, new_entry)?;
                    new_entry
                },
            };
            let pml1 = PageMap::new(entry.address, PageMapLevel::L1)?;
            //Map within PML1
            if position == start_position && position == end_position-1 {
                pml1.map_pages_offset_pml1(physical_offset, start_offset, size, write, supervisor, execute_disable)?;
            }
            else if position == start_position {
                pml1.map_pages_offset_pml1(physical_offset, start_offset, PAGE_SIZE_2MIB-start_offset, write, supervisor, execute_disable)?;
            }
            else if position == end_position-1 {
                pml1.map_pages_offset_pml1(unsafe {((physical_offset as *mut u8).add(position*PAGE_SIZE_2MIB - start_offset)) as *mut u64}, 0, end_size, write, supervisor, execute_disable)?;
            }
            else {
                pml1.map_pages_offset_pml1(unsafe {((physical_offset as *mut u8).add(position*PAGE_SIZE_2MIB - start_offset)) as *mut u64}, 0, PAGE_SIZE_2MIB, write, supervisor, execute_disable)?;
            }
        }
        //Return
        Ok(())
    }
    fn map_pages_offset_pml3(&self, page_allocator: &dyn PageAllocator, physical_offset: *mut u64, map_offset: usize, size: usize, write: bool, supervisor: bool, execute_disable: bool) -> Result<(), &'static str> {
        //Check Parameters
        if self.map_level                            != PageMapLevel::L3 {return Err("Page Map: Offset PML2 called on page map of wrong level.")}
        if physical_offset as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Physical offset not aligned to 4KiB boundary.")}
        if map_offset      as usize % PAGE_SIZE_4KIB != 0                {return Err("Page Map: Map offset not aligned to 4KiB boundary.")}
        if map_offset      +  size  > PAGE_SIZE_512G                     {return Err("Page Map: Map position and size requested does not fit within level 2 map boundaries.")}
        //Position Variables
        let start_position: usize =  map_offset         / PAGE_SIZE_1GIB;
        let start_offset:   usize =  map_offset         % PAGE_SIZE_1GIB;
        let end_size:       usize = (map_offset + size) % PAGE_SIZE_1GIB;
        let end_position:   usize = (map_offset + size) / PAGE_SIZE_1GIB + if end_size != 0 {1} else {0};
        //Loop
        for position in start_position..end_position {
            //Retrieve PML2
            let entry = match self.read_entry(position) {
                Ok(entry) => {
                    if entry.present {
                        entry
                    }
                    else {
                        let new_entry = PageMapEntry::new(PageMapLevel::L3, PageMapEntryType::Table, page_allocator.allocate_page(), write, supervisor, execute_disable)?;
                        self.write_entry(position, new_entry)?;
                        new_entry
                    }
                },
                Err(_) => {
                    let new_entry = PageMapEntry::new(PageMapLevel::L3, PageMapEntryType::Table, page_allocator.allocate_page(), write, supervisor, execute_disable)?;
                    self.write_entry(position, new_entry)?;
                    new_entry
                },
            };
            let pml2 = PageMap::new(entry.address, PageMapLevel::L2)?;
            //Map within PML2
            if position == start_position && position == end_position-1 {
                pml2.map_pages_offset_pml2(page_allocator, physical_offset, start_offset, size, write, supervisor, execute_disable)?;
            }
            else if position == start_position {
                pml2.map_pages_offset_pml2(page_allocator, physical_offset, start_offset, PAGE_SIZE_2MIB-start_offset, write, supervisor, execute_disable)?;
            }
            else if position == end_position-1 {
                pml2.map_pages_offset_pml2(page_allocator, unsafe {((physical_offset as *mut u8).add(position*PAGE_SIZE_2MIB - start_offset)) as *mut u64}, 0, end_size, write, supervisor, execute_disable)?;
            }
            else {
                pml2.map_pages_offset_pml2(page_allocator, unsafe {((physical_offset as *mut u8).add(position*PAGE_SIZE_2MIB - start_offset)) as *mut u64}, 0, PAGE_SIZE_2MIB, write, supervisor, execute_disable)?;
            }
        }
        //Return
        Ok(())
    }
}

//Page Table Entry
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct PageMapEntry {
    pub entry_level:     PageMapLevel,
    pub entry_type:      PageMapEntryType, //Bit 7 in some cirumstances, indicates page refers to memory when it could refer to a table
    pub address:         *mut u64,
    pub present:         bool, //ALL: Bit 0, indicates entry exists
    pub write:           bool, //ALL: Bit 1, indicates page may be written to
    pub supervisor:      bool, //ALL: Bit 2, indicates page can only be accessed in Ring 0
    pub write_through:   bool, //ALL: Bit 3, something about how memory access works
    pub cache_disable:   bool, //ALL: Bit 4, something else about how memory access works
    pub accessed:        bool, //ALL: Bit 5, indicates page has been accessed
    pub dirty:           Option<bool>, //MEMORY: Bit 6, indicates page has been written to
    pub attribute_table: Option<bool>, //MEMORY: Bit 7 (L1) or Bit 12 (L2, L3), indicates yet another thing about how memory access works
    pub global:          Option<bool>, //MEMORY: Bit 8,
    pub execute_disable: bool, //ALL: Bit 63, indicates code may not be executed from this page
}
impl PageMapEntry {
    //Read from u64, intended to read a page table entry from RAM
    pub fn from_u64(data: u64, entry_level: PageMapLevel) -> Result<Self, &'static str> {
        let entry_type = {
            if      entry_level == PageMapLevel::L5 || entry_level == PageMapLevel::L4 {PageMapEntryType::Table}
            else if entry_level == PageMapLevel::L3 || entry_level == PageMapLevel::L2 { 
                if data & (1<<7) > 0                                                   {PageMapEntryType::Memory}
                else                                                                   {PageMapEntryType::Table}}
            else                                                                       {PageMapEntryType::Memory}
        };
        Ok(Self {
            entry_level,
            entry_type,
            address: match (entry_level, entry_type) {
                (PageMapLevel::L5, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L4, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L3, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L2, PageMapEntryType::Table)  =>      data & 0o_000_777_777_777_777_777_0000_u64,
                (PageMapLevel::L3, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_000_000_0000_u64,
                (PageMapLevel::L2, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_777_000_0000_u64,
                (PageMapLevel::L1, PageMapEntryType::Memory) =>      data & 0o_000_777_777_777_777_777_0000_u64,
                _ => {return Err("Page Table Entry: Invalid combination of level and entry type found.")}
            } as *mut u64,
            present:                                                 data & (1<<0o00) > 0,
            write:                                                   data & (1<<0o01) > 0,
            supervisor:                                              data & (1<<0o02) > 0,
            write_through:                                           data & (1<<0o03) > 0,
            cache_disable:                                           data & (1<<0o04) > 0,
            accessed:                                                data & (1<<0o05) > 0,
            dirty: match entry_type {
                                PageMapEntryType::Memory     => Some(data & (1<<0o06) > 0),
                                PageMapEntryType::Table      => None,
            },
            attribute_table: match (entry_level, entry_type) {
                (PageMapLevel::L3, PageMapEntryType::Memory) => Some(data & (1<<0o14) > 0),
                (PageMapLevel::L2, PageMapEntryType::Memory) => Some(data & (1<<0o14) > 0),
                (PageMapLevel::L1, PageMapEntryType::Memory) => Some(data & (1<<0o07) > 0),
                _                                            => None,
            },
            global: match entry_type {
                                PageMapEntryType::Memory     => Some(data & (1<<0o10) > 0),
                                PageMapEntryType::Table      => None,
            },
            execute_disable:                                         data & (1<<0o77) > 0,
        })
    }
    
    //Convert to u64, intended to aid in writing a page table entry into RAM
    pub fn to_u64(&self) -> Result<u64, &'static str> {
        let mut result: u64 = 0;
        result |= match (self.entry_level, self.entry_type) {
            (PageMapLevel::L5, PageMapEntryType::Table)  => self.address as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L4, PageMapEntryType::Table)  => self.address as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L3, PageMapEntryType::Table)  => self.address as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L2, PageMapEntryType::Table)  => self.address as u64 & 0o_000_777_777_777_777_777_0000_u64,
            (PageMapLevel::L3, PageMapEntryType::Memory) => self.address as u64 & 0o_000_777_777_777_000_000_0000_u64,
            (PageMapLevel::L2, PageMapEntryType::Memory) => self.address as u64 & 0o_000_777_777_777_777_000_0000_u64,
            (PageMapLevel::L1, PageMapEntryType::Memory) => self.address as u64 & 0o_000_777_777_777_777_777_0000_u64,
            _ => {return Err("Page Table Entry: Invalid combination of level and entry type in struct.")}
        };
        if self.present       {result |= 1<<0o00}
        if self.write         {result |= 1<<0o01}
        if self.supervisor    {result |= 1<<0o02}
        if self.write_through {result |= 1<<0o03}
        if self.cache_disable {result |= 1<<0o04}
        if self.accessed      {result |= 1<<0o05}
        if self.entry_type == PageMapEntryType::Memory {
            if self.dirty.is_some() && self.dirty.unwrap() {result |= 1<<0o06}
            if self.entry_level == PageMapLevel::L3 || self.entry_level == PageMapLevel::L2 {
                result |= 1<<0o07;
                if self.attribute_table.is_some() && self.attribute_table.unwrap() {result |= 1<<0o14}
            }
            else if self.entry_level == PageMapLevel::L1 && self.attribute_table.is_some() && self.attribute_table.unwrap() {result |= 1<<0o07}
        }
        if self.execute_disable {result |= 1<<0o77}
        Ok(result)
    }

    //New
    pub fn new(entry_level: PageMapLevel, entry_type: PageMapEntryType, address: *mut u64, write: bool, supervisor: bool, execute_disable: bool) -> Result<Self, &'static str> {
        match (entry_level, entry_type) {
            (PageMapLevel::L5, PageMapEntryType::Table)  => {if address as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            (PageMapLevel::L4, PageMapEntryType::Table)  => {if address as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            (PageMapLevel::L3, PageMapEntryType::Table)  => {if address as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            (PageMapLevel::L2, PageMapEntryType::Table)  => {if address as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            (PageMapLevel::L3, PageMapEntryType::Memory) => {if address as usize % PAGE_SIZE_1GIB != 0 {return Err("Page Table Entry: Address is not aligned to a 1GiB boundary.")}},
            (PageMapLevel::L2, PageMapEntryType::Memory) => {if address as usize % PAGE_SIZE_2MIB != 0 {return Err("Page Table Entry: Address is not aligned to a 2MiB boundary.")}},
            (PageMapLevel::L1, PageMapEntryType::Memory) => {if address as usize % PAGE_SIZE_4KIB != 0 {return Err("Page Table Entry: Address is not aligned to a 4KiB boundary.")}},
            _ => {return Err("Page Table Entry: Invalid combination of level and entry type in constructor.")}
        };
        Ok(Self {
            entry_level,
            entry_type,
            address,
            present:         true,
            write,
            supervisor,
            write_through:   false,
            cache_disable:   false,
            accessed:        false,
            dirty:           if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            attribute_table: if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            global:          if entry_type == PageMapEntryType::Memory {Some(false)} else {None},
            execute_disable,
        })
    }
}

//Page Map Level
numeric_enum! {
    #[repr(u8)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum PageMapLevel {
        L5 = 5,
        L4 = 4,
        L3 = 3,
        L2 = 2,
        L1 = 1,
    }
}

//Page Map Entry Type
#[derive(PartialEq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum PageMapEntryType {
    Memory,
    Table,
}
