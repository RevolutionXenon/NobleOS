// GLUON
// Gluon is the Noble loading library:
// Memory locations of important objects
// Sizes and counts related to page tables
// Macros, Structs, and Enums related to the contents of ELF files


// HEADER
//Flags
#![no_std]

//Imports
use core::convert::{TryFrom};
use elf_file_header::ELFFileHeader;
use elf_program_header::ELFProgramHeader;

//Constants
pub const GLUON_VERSION:  &    str   = "vDEV-2021-08-12";                             //CURRENT VERSION OF GRAPHICS LIBRARY
//                                          SIGN PM4 PM3 PM2 PM1 OFFSET
pub const PHYSM_PHYS_OCT:      usize = 0o________000__________________usize;          //PHYSICAL MEMORY PHYSICAL LOCATION PML4 OFFSET
pub const PHYSM_PHYS_PTR: *mut u8    = 0o_000000_000_000_000_000_0000_u64 as *mut u8; //PHYSICAL MEMORY PHYSICAL LOCATION POINTER
pub const KERNL_VIRT_OCT:      usize = 0o________400__________________usize;          //KERNEL VIRTUAL LOCATION PML4 TABLE OFFSET
pub const KERNL_VIRT_PTR: *mut u8    = 0o_177777_400_000_000_000_0000_u64 as *mut u8; //KERNEL VIRTUAL LOCATION POINTER
pub const FRAME_VIRT_OCT:      usize = 0o________775__________________usize;          //FRAME BUFFER VIRTUAL LOCATION PML4 OFFSET
pub const FRAME_VIRT_PTR: *mut u8    = 0o_177777_775_000_000_000_0000_u64 as *mut u8; //FRAME BUFFER VIRTUAL LOCATION POINTER
pub const PHYSM_VIRT_OCT:      usize = 0o________776__________________usize;          //PHYSICAL MEMORY VIRTUAL LOCATION PML4 OFFSET
pub const PHYSM_VIRT_PTR: *mut u8    = 0o_177777_776_000_000_000_0000_u64 as *mut u8; //PHYSICAL MEMORY VIRTUAL LOCATION POINTER
pub const PGMAP_VIRT_OCT:      usize = 0o________777__________________usize;          //PAGE MAP VIRTUAL LOCATION PML4 OFFSET
pub const PGMAP_VIRT_PTR: *mut u8    = 0o_177777_777_000_000_000_0000_u64 as *mut u8; //PAGE MAP VIRTUAL LOCATION POINTER
pub const PAGE_SIZE_4KIB:      usize = 0o______________________1_0000_usize;          //MEMORY PAGE SIZE (  4KiB),                            PAGE MAP LEVEL 1 ENTRY SIZE
pub const PAGE_SIZE_2MIB:      usize = 0o__________________1_000_0000_usize;          //MEMORY PAGE SIZE (  2MiB), PAGE MAP LEVEL 1 CAPACITY, PAGE MAP LEVEL 2 ENTRY SIZE
pub const PAGE_SIZE_1GIB:      usize = 0o______________1_000_000_0000_usize;          //MEMORY PAGE SIZE (  1GiB), PAGE MAP LEVEL 2 CAPACITY, PAGE MAP LEVEL 3 ENTRY SIZE
pub const PAGE_SIZE_512G:      usize = 0o__________1_000_000_000_0000_usize;          //MEMORY PAGE SIZE (512GiB), PAGE MAP LEVEL 3 CAPACITY
pub const PAGE_SIZE_256T:      usize = 0o______1_000_000_000_000_0000_usize;          //MEMORY PAGE SIZE (256TiB), PAGE MAP LEVEL 4 CAPACITY
pub const PAGE_NMBR_LVL1:      usize = 0o________________________1000_usize;          //NUMBER OF PAGE TABLE ENTRIES 1 LEVEL UP
pub const PAGE_NMBR_LVL2:      usize = 0o____________________100_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 2 LEVELS UP
pub const PAGE_NMBR_LVL3:      usize = 0o________________100_000_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 3 LEVELS UP
pub const PAGE_NMBR_LVL4:      usize = 0o____________100_000_000_0000_usize;          //NUMBER OF PAGE TABLE ENTRIES 4 LEVELS UP


// MACROS
//Numeric Enum
macro_rules!numeric_enum {
    (
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
    fn read(&mut self, offset: u64, buffer: &mut [u8]) -> Result<(), &'static str>;
}


// STRUCTS
//Full ELF File Header
pub struct ELFFile<'a> {
    pub file:            &'a mut dyn LocationalRead,
    pub file_header:                 ELFFileHeader,
    pub program_headers: &'a         [ELFProgramHeader],
}
impl<'a> ELFFile<'a>{
    // CONSTRUCTOR
    pub fn new(file: &'a mut dyn LocationalRead, program_header_buffer: &'a mut [ELFProgramHeader]) -> Result<ELFFile<'a>, &'static str> {
        //Load File Header
        let file_header = match ELFFileHeader::new(&{let mut buf:[u8; 0x40] = [0u8; 0x40]; match file.read(0x00,  &mut buf) {Ok(_) => (), Err(error) => return Err(error)}; buf}) {Ok(result) => result, Err(error) => return Err(error)};
        //Load Program Headers
        if file_header.program_header_number as usize > program_header_buffer.len() {return Err("ELF File: More than the given maximum number of program headers found.")}
        for i in 0..file_header.program_header_number as usize {
            //New Program Header
            program_header_buffer[i] = match ELFProgramHeader::new(&{let mut buf:[u8; 0x38] = [0u8; 0x38]; match file.read((file_header.header_size + i as u16 *file_header.program_header_entry_size) as u64, &mut buf) {Ok(_) => (), Err(error) => return Err(error)}; buf}, file_header.bit_width, file_header.endianness) {
                Ok(valid_program_header)    => valid_program_header,
                Err(invalid_program_header) => invalid_program_header
            }
        }
        //Return
        return Ok(ELFFile {
            file,
            file_header,
            program_headers: & program_header_buffer[0..file_header.program_header_number as usize],
        });
    }

    // FUNCTIONS
    //Total memory size of program from lowest virtual address to highest virtual address
    pub fn memory_size(&self) -> u64 {
        //Buffers
        let mut program_lowest_address:  u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let mut program_highest_address: u64 = 0x0000_0000_0000_0000;
        let mut loadable_found: bool = false;
        //Loop over program headers
        for program_header in self.program_headers {
            //Check if program segment is loadable
            if program_header.program_type == elf_program_header::ProgramType::Loadable {
                loadable_found = true;
                //Check if minimum virtual address needs adjusting
                if program_header.virtual_address < program_lowest_address {
                    program_lowest_address = program_header.virtual_address;
                }
                //Check if maximum virtual address needs adjusting
                if program_header.virtual_address + program_header.memory_size > program_highest_address {
                    program_highest_address = program_header.virtual_address + program_header.memory_size;
                }
            }
        }
        //Return
        return if loadable_found {program_highest_address - program_lowest_address} else {0};
    }
}


// MODULES
//ELF File Header
pub mod elf_file_header {
    // IMPORTS
    use core::convert::{TryFrom, TryInto};
    use crate::{BitWidth, Endianness};

    // STRUCTS
    //File Header
    #[derive(Clone, Copy)]
    pub struct ELFFileHeader {
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
    impl ELFFileHeader {
        // CONSTRUCTOR
        pub fn new(bytes: &[u8]) -> Result<ELFFileHeader, &'static str> {
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
            return Result::Ok(ELFFileHeader {
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
        pub enum IdentVersion {
            Original = 0x01,
        }
    }
    
    //Application Binary Interface
    numeric_enum! {
        #[repr(u8)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
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
        pub enum Version {
            Original = 0x01,
        }
    }
}

//ELF Program Header
pub mod elf_program_header {
    // IMPORTS
    use core::convert::{TryFrom, TryInto};
    use crate::{BitWidth, Endianness, elf_file_header::ELFFileHeader};

    // STRUCTS
    //Program Header
    #[derive(Clone, Copy)]
    pub struct ELFProgramHeader {
        pub program_type:        ProgramType,
        pub flags:               [u8;4],
        pub file_offset:         u64,
        pub virtual_address:     u64,
        pub physical_address:    u64,
        pub file_size:           u64,
        pub memory_size:         u64,
        pub alignment:           u64,
        pub diagnostic: &'static str,
    }
    pub fn default_diagnostic(diagnostic: &'static str) -> ELFProgramHeader{
        ELFProgramHeader {
            program_type:     ProgramType::Null,
            flags:            [0;4],
            file_offset:      0,
            virtual_address:  0,
            physical_address: 0,
            file_size:        0,
            memory_size:      0,
            alignment:        0,
            diagnostic:       diagnostic,
        }
    }
    impl ELFProgramHeader {
        // CONSTRUCTOR
        //New
        pub fn new(head: &[u8], bit_width: BitWidth, endianness: Endianness) -> Result<ELFProgramHeader, ELFProgramHeader> {
            let diagnostic: &str;
            if bit_width == BitWidth::W64 && head.len() != 0x38 {return Err(default_diagnostic("ELF Program Header: Length of data given to parse from incorrect."))};
            let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
                Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
                Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
            };
            return Result::Ok(ELFProgramHeader {
                program_type:     ProgramType::try_from(u32_fb(head[0x00..0x04].try_into().unwrap())).map_err(|_| default_diagnostic("ELF Program Header: Length of data given to parse from incorrect."))?,
                flags:            head[0x04..0x08].try_into().unwrap(),
                file_offset:      u64_fb(head[0x08..0x10].try_into().unwrap()),
                virtual_address:  u64_fb(head[0x10..0x18].try_into().unwrap()),
                physical_address: u64_fb(head[0x18..0x20].try_into().unwrap()),
                file_size:        u64_fb(head[0x20..0x28].try_into().unwrap()),
                memory_size:      u64_fb(head[0x28..0x30].try_into().unwrap()),
                alignment:        u64_fb(head[0x30..0x38].try_into().unwrap()),
                diagnostic:       "ELF Program Header: Valid.",
            })
        }
    }
    impl Default for ELFProgramHeader {
        fn default() -> Self {
            default_diagnostic("ELF Program Header: Default unitialized value.")
        }
    }

    // ENUMS
    //Program Type
    numeric_enum! {
        #[repr(u32)]
        #[derive(PartialEq)]
        #[derive(Clone, Copy)]
        pub enum ProgramType {
            Null                 = 0x00,
            Loadable             = 0x01,
            Dynamic              = 0x02,
            Interpreter          = 0x03,
            Note                 = 0x04,
            ProgramHeader        = 0x06,
            ThreadLocalStorage   = 0x07,
        }
    }
}


// ENUMS
//Bit Width
numeric_enum! {
    #[repr(u8)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
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
    pub enum Endianness {
        Little = 0x01,
        Big    = 0x02,
    }
}
