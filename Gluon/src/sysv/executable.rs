// GLUON: SYSTEM V EXECUTABLE
// Structs and enums related to the contents and handling of System V object files (ELF files)


// HEADER
//Imports
use crate::numeric_enum;
use crate::noble::file_system::Volume;
use crate::noble::return_code::ReturnCode;
use core::convert::{TryFrom, TryInto};
use core::intrinsics::copy_nonoverlapping;
use core::ptr::write_volatile;


// ELF FILES
//Full ELF File Handling Routines
#[derive(Debug)]
pub struct ELFFile<'a, RO: 'a+Volume> {
    pub file: &'a RO,
    pub header:   Header,
}
impl<'a, RO: 'a+Volume> ELFFile<'a, RO> {
    // CONSTRUCTOR
    pub fn new(file: &'a RO) -> Result<ELFFile<'a, RO>, ReturnCode> {
        //Load File Header
        let header = Header::new(&{
            let mut buffer:[u8; 0x40] = [0u8; 0x40];
            file.read_all(0x00, &mut buffer)?;
            buffer
        })?;
        //Return
        Ok(ELFFile {
            file,
            header,
        })
    }

    // ITERATORS
    pub fn programs(&self) -> ProgramIterator<RO> {
        ProgramIterator::new(self.file, &self.header)
    }

    pub fn sections(&self) -> SectionIterator<RO> {
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
            if program.program_type == ProgramType::Loadable {
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
    pub unsafe fn load(&mut self, location: *mut u8) -> Result<(), ReturnCode> {
        for position in 0..self.program_memory_size() as usize {
            write_volatile(location.add(position), 0x00);
        }
        let program_iterator = ProgramIterator::new(self.file, &self.header)
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .filter(|program| program.program_type == ProgramType::Loadable);
        for program in program_iterator {
            const BUFFER_SIZE: usize = 512;
            let mut buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];
            let count: u64 = program.file_size as u64/BUFFER_SIZE as u64;
            for file_positon in 0..count {
                self.file.read_all(program.file_offset as u64 +file_positon*BUFFER_SIZE as u64, &mut buffer)?;
                copy_nonoverlapping(buffer.as_ptr(), location.add(program.virtual_address as usize + file_positon as usize * BUFFER_SIZE), BUFFER_SIZE);
            }
            let leftover: usize = program.file_size as usize %BUFFER_SIZE;
            if leftover != 0 {
                self.file.read_all(program.file_offset as u64 + count*BUFFER_SIZE as u64, &mut buffer[0..leftover])?;
                copy_nonoverlapping(buffer.as_ptr(), location.add(program.virtual_address as usize + count as usize * BUFFER_SIZE), leftover);
            }
        }
        //Return
        Ok(())
    }

    //Do Relocation (Very Important to Load First) **NOT FINISHED**
    pub unsafe fn relocate(&mut self, loaded_location: *mut u8, reloc_location: *mut u8) -> Result<(), ReturnCode> {
        //Ensure correct object type
        if self.header.object_type != ObjectType::Shared {return Err(ReturnCode::UnsupportedFeature)}
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
                            _ => {return Err(ReturnCode::UnsupportedFeature)},
                        }
                    }
                }
                //Return
                Ok(())
            },
            //Return Error
            _ => Err(ReturnCode::UnsupportedFeature)
        }
    }
}


// ELF FILE HEADER
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
    pub fn new(bytes: &[u8]) -> Result<Header, ReturnCode> {
        if bytes.len()       <  0x10                             {return Err(ReturnCode::BufferTooSmall)}
        if bytes[0x00..0x04] != [0x7Fu8, 0x45u8, 0x4cu8, 0x46u8] {return Err(ReturnCode::InvalidIdentifier)}
        if bytes[0x04]       != 0x02                             {return Err(ReturnCode::UnsupportedFeature)}
        if bytes.len()       <  0x40                             {return Err(ReturnCode::BufferTooSmall)}
        if bytes[0x09..0x10] != [0x00u8; 7]                      {return Err(ReturnCode::InvalidData)}
        let endianness:Endianness = Endianness::try_from(bytes[0x05]).map_err(|_| ReturnCode::InvalidData)?;
        let (u16_fb, u32_fb, u64_fb): (fn([u8;2])->u16, fn([u8;4])->u32, fn([u8;8])->u64) = match endianness {
            Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
            Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
        };
        Result::Ok(Header {
            bit_width:                                   BitWidth::try_from(       bytes[0x04]                                                            ).map_err(|_| ReturnCode::InvalidData)?,
            endianness,
            ident_version:                           IdentVersion::try_from(       bytes[0x06]                                                            ).map_err(|_| ReturnCode::InvalidData)?,
            binary_interface:          ApplicationBinaryInterface::try_from(       bytes[0x07]                                                            ).map_err(|_| ReturnCode::InvalidData)?,
            binary_interface_version:                                              bytes[0x08],
            object_type:                               ObjectType::try_from(u16_fb(bytes[0x10..0x12].try_into().map_err(|_| ReturnCode::SlicingError)?)).map_err(|_| ReturnCode::InvalidData)?,
            architecture:              InstructionSetArchitecture::try_from(u16_fb(bytes[0x12..0x14].try_into().map_err(|_| ReturnCode::SlicingError)?)).map_err(|_| ReturnCode::InvalidData)?,
            version:                                      Version::try_from(u32_fb(bytes[0x14..0x18].try_into().map_err(|_| ReturnCode::SlicingError)?)).map_err(|_| ReturnCode::InvalidData)?,
            entry_point:                                                    u64_fb(bytes[0x18..0x20].try_into().map_err(|_| ReturnCode::SlicingError)?),
            program_header_offset:                                    match u64_fb(bytes[0x20..0x28].try_into().map_err(|_| ReturnCode::SlicingError)?) {0x40 => 0x40, _ => return Err(ReturnCode::InvalidData)},
            section_header_offset:                                          u64_fb(bytes[0x28..0x30].try_into().map_err(|_| ReturnCode::SlicingError)?),
            flags:                                                                 bytes[0x30..0x34].try_into().map_err(|_| ReturnCode::SlicingError)?,
            header_size:                                              match u16_fb(bytes[0x34..0x36].try_into().map_err(|_| ReturnCode::SlicingError)?) {0x40 => 0x40, _ => return Err(ReturnCode::InvalidData)},
            program_header_entry_size:                                match u16_fb(bytes[0x36..0x38].try_into().map_err(|_| ReturnCode::SlicingError)?) {0x38 => 0x38, _ => return Err(ReturnCode::InvalidData)},
            program_header_number:                                          u16_fb(bytes[0x38..0x3A].try_into().map_err(|_| ReturnCode::SlicingError)?),
            section_header_entry_size:                                match u16_fb(bytes[0x3A..0x3C].try_into().map_err(|_| ReturnCode::SlicingError)?) {0x40 => 0x40, _ => return Err(ReturnCode::InvalidData)},
            section_header_number:                                          u16_fb(bytes[0x3C..0x3E].try_into().map_err(|_| ReturnCode::SlicingError)?),
            string_section_index:                             {let a: u16 = u16_fb(bytes[0x3E..0x40].try_into().map_err(|_| ReturnCode::SlicingError)?);
                                                                     if a < u16_fb(bytes[0x3C..0x3E].try_into().map_err(|_| ReturnCode::SlicingError)?) {a} else {return Err(ReturnCode::InvalidData)}},
        })
    }
}

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
        ARMEABI           = 0x40,
        ARM               = 0x61,
        Standalone        = 0xFF,
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
        None            = 0x0000, EmM32           = 0x0001, EmSparc         = 0x0002, Em386           = 0x0003,
        Em68K           = 0x0004, Em88K           = 0x0005, EmIamcu         = 0x0006, Em860           = 0x0007,
        EmMips          = 0x0008, EmS370          = 0x0009, EmMipsRS3LE     = 0x000A, /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              EmPaRisc        = 0x000F,
        /*Reserved*/              EmVPP500        = 0x0011, EmSparc32Plus   = 0x0012, Em960           = 0x0013,
        EmPPC           = 0x0014, EmPPC64         = 0x0015, EmS390          = 0x0016, /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        EmV800          = 0x0024, EmFR20          = 0x0025, EmRH32          = 0x0026, EmRCE           = 0x0027,
        EmARM           = 0x0028, EmAlpha         = 0x0029, EmSH            = 0x002A, EmSparcV9       = 0x002B,
        EmTriCore       = 0x002C, EmARC           = 0x002D, EmH8300         = 0x002E, EmH8300H        = 0x002F,
        EmH8S           = 0x0030, EmH8500         = 0x0031, EmIA64          = 0x0032, EmMipsX         = 0x0033,
        EmColdFire      = 0x0034, Em68HC12        = 0x0035, EmMMA           = 0x0036, EmPCP           = 0x0037,
        EmNCPU          = 0x0038, EmNDR1          = 0x0039, EmStarCore      = 0x003A, EmME16          = 0x003B,
        EmST100         = 0x003C, EmTinyJ         = 0x003D, EmX86_64        = 0x003E, EmPDSP          = 0x003F,
        EmPDP10         = 0x0040, EmPDP11         = 0x0041, EmFX66          = 0x0042, EmST9Plus       = 0x0043,
        EmST7           = 0x0044, Em68HC16        = 0x0045, Em68HC11        = 0x0046, Em68HC08        = 0x0047,
        Em68HC05        = 0x0048, EmSVx           = 0x0049, EmST19          = 0x004A, EmVAX           = 0x004B,
        EmCRIS          = 0x004C, EmJavelin       = 0x004D, EmFirepath      = 0x004E, EmZSP           = 0x004F,
        EmMMIX          = 0x0050, EmHUANY         = 0x0051, EmPrism         = 0x0052, EmAVR           = 0x0053,
        EmFR30          = 0x0054, EmD10V          = 0x0055, EmD30V          = 0x0056, EmV850          = 0x0057,
        EmM32R          = 0x0058, EmMN10300       = 0x0059, EmMN10200       = 0x005A, EmPJ            = 0x005B,
        EmOpenRISC      = 0x005C, EmARCA5         = 0x005D, EmXtensa        = 0x005E, EmVideoCore     = 0x005F,
        EmTMMGPP        = 0x0060, EmNS32K         = 0x0061, EmTPC           = 0x0062, EmSNP1K         = 0x0063,
        EmST200         = 0x0064, EmIP2K          = 0x0065, EmMAX           = 0x0066, EmCR            = 0x0067,
        EmF2MC16        = 0x0068, EmMsp430        = 0x0069, EmBlackfin      = 0x006A, EmSEC33         = 0x006B,
        EmSEP           = 0x006C, EmArca          = 0x006D, EmUnicore       = 0x006E, EmExcess        = 0x006F,
        EmDXP           = 0x0070, EmAlteraNoisII  = 0x0071, EmCRX           = 0x0072, EmXGATE         = 0x0073,
        EmC166          = 0x0074, EmM16C          = 0x0075, EmDsPIC30F      = 0x0076, EmCE            = 0x0077,
        EmM32C          = 0x0078, /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              EmTSK3000       = 0x0083,
        EmRS08          = 0x0084, EmSHARC         = 0x0085, EmECOG2         = 0x0086, EmScore7        = 0x0087,
        EmDSP24         = 0x0088, EmVideoCore3    = 0x0089, EmLatticeMico32 = 0x008A, EmSEC17         = 0x008B,
        EmTIC6000       = 0x008C, EmTIC2000       = 0x008D, EmTIC5500       = 0x008E, EmTIARP32       = 0x008F,
        EmTIPRU         = 0x0090, /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        EmMMDSPPlus     = 0x00A0, EmCypressM8C    = 0x00A1, EmR32C          = 0x00A2, EmTriMedia      = 0x00A3,
        EmQDSP6         = 0x00A4, Em8051          = 0x00A5, EmSTxP7x        = 0x00A6, EmNDS32         = 0x00A7,
        EmECOG1X        = 0x00A8, EmMAXQ30        = 0x00A9, EmXIMO16        = 0x00AA, EmManik         = 0x00AB,
        EmCrayNV2       = 0x00AC, EmRX            = 0x00AD, EmMETAG         = 0x00AE, EmMCSTElbrus    = 0x00AF,
        EmECOG16        = 0x00B0, EmCR16          = 0x00B1, EmETPU          = 0x00B2, EmSLE9X         = 0x00B3,
        EmL10M          = 0x00B4, EmK10M          = 0x00B5, /*Reserved*/              EmAARCH64       = 0x00B7,
        /*Reserved*/              EmAVR32         = 0x00B9, EmSTM8          = 0x00BA, EmTILE64        = 0x00BB,
        EmTILEPro       = 0x00BC, EmMicroBlaze    = 0x00BD, EmCUDA          = 0x00BE, EmTILEGx        = 0x00BF,
        EmCloudShield   = 0x00C0, EmCoreA1st      = 0x00C1, EmCoreA2nd      = 0x00C2, EmARCCompact2   = 0x00C3,
        EmOpen8         = 0x00C4, EmRL78          = 0x00C5, EmVideoCore5    = 0x00C6, Em78KOR         = 0x00C7,
        Em56800EX       = 0x00C8, EmBA1           = 0x00C9, EmBA2           = 0x00CA, EmXCORE         = 0x00CB,
        EmMCHPPIC       = 0x00CC, /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              EmKM32          = 0x00D2, EmKMX32         = 0x00D3,
        EmEMX16         = 0x00D4, EmEMX8          = 0x00D5, EmKVARC         = 0x00D6, EmCDP           = 0x00D7,
        EmCOGE          = 0x00D8, EmCool          = 0x00D9, EmNORC          = 0x00DA, EmCSRKalimba    = 0x00DB,
        EmZ80           = 0x00DC, EmVISIUM        = 0x00DD, EmFT32          = 0x00DE, EmMoxie         = 0x00DF,
        EmAMDGPU        = 0x00E0, /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              EmRISCV         = 0x00F3,
        /*Reserved*/              /*Reserved*/              /*Reserved*/              EmBPF           = 0x00F7,
        EmCSKY          = 0x00F8, /*Reserved*/              /*Reserved*/              /*Reserved*/             
        /*Reserved*/              /*Reserved*/              /*Reserved*/              /*Reserved*/             
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


// ELF PROGRAM HEADER
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
    pub fn new(data: &[u8], bit_width: BitWidth, endianness: Endianness) -> Result<Self, ReturnCode> {
        let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
            Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
            Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
        };
        match bit_width {
            BitWidth::W32 => {
                if data.len() != 0x20 {return Err(ReturnCode::IncorrectBufferLength)}
                Ok(Self {
                    program_type:     ProgramType::try_from(
                                      u32_fb(data[0x00..0x04].try_into().map_err( |_| ReturnCode::SlicingError)?))
                                                                        .map_err( |_| ReturnCode::InvalidData)?,
                    file_offset:      u64_fb(data[0x04..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    virtual_address:  u64_fb(data[0x08..0x0C].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    physical_address: u64_fb(data[0x0C..0x10].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    file_size:        u64_fb(data[0x20..0x28].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    memory_size:      u64_fb(data[0x28..0x30].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    flags:            u32_fb(data[0x04..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    alignment:        u64_fb(data[0x30..0x38].try_into().map_err( |_| ReturnCode::SlicingError)?),
                })
            },
            BitWidth::W64 => {
                if data.len() != 0x38 {return Err(ReturnCode::IncorrectBufferLength)};
                Ok(Self {
                    program_type:     ProgramType::try_from(
                                      u32_fb(data[0x00..0x04].try_into().map_err( |_| ReturnCode::SlicingError)?))
                                                                        .map_err( |_| ReturnCode::InvalidData)?,
                    flags:            u32_fb(data[0x04..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    file_offset:      u64_fb(data[0x08..0x10].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    virtual_address:  u64_fb(data[0x10..0x18].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    physical_address: u64_fb(data[0x18..0x20].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    file_size:        u64_fb(data[0x20..0x28].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    memory_size:      u64_fb(data[0x28..0x30].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    alignment:        u64_fb(data[0x30..0x38].try_into().map_err( |_| ReturnCode::SlicingError)?),
                })
            }, 
        }
    }
}

//Program Header Iterator
pub struct ProgramIterator<'a, RO: 'a+Volume> {
    file:       &'a RO,
    bit_width:      BitWidth,
    endianness:     Endianness,
    base_offset:    u64,
    entry_position: usize,
    entry_count:    usize,
}
impl<'a, RO: 'a+Volume> ProgramIterator<'a, RO> {
    // FUNCTIONS
    //Constructor
    pub fn new(file: &'a RO, file_header: &Header) -> Self{
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
    fn entry(&mut self) -> Result<Program, ReturnCode> {
        match self.bit_width {
            BitWidth::W32 => {
                let mut buffer: [u8; 0x20] = [0u8; 0x20];
                self.file.read_all(self.base_offset as u64 + 0x20*self.entry_position as u64, &mut buffer)?;
                Program::new(&buffer, self.bit_width, self.endianness)
            },
            BitWidth::W64 => {
                let mut buffer: [u8; 0x38] = [0u8; 0x38];
                self.file.read_all(self.base_offset as u64 + 0x38*self.entry_position as u64, &mut buffer)?;
                Program::new(&buffer, self.bit_width, self.endianness)
            },
        }
    }
}
impl<'a, RO: 'a+Volume> Iterator for ProgramIterator<'a, RO> {
    type Item = Result<Program, ReturnCode>;
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


// ELF SECTION HEADER
//Section Header Iterator
pub struct SectionIterator<'a, RO: 'a+Volume> {
    file:   &'a     RO,
    bit_width:      BitWidth,
    endianness:     Endianness,
    base_offset:    u64,
    entry_position: usize,
    entry_count:    usize,
}
impl<'a, RO: 'a+Volume> SectionIterator<'a, RO> {
    // FUNCTIONS
    //Constructor
    pub fn new(file: &'a RO, file_header: &Header) -> Self{
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
    fn entry(&mut self) -> Result<Section, ReturnCode> {
        match self.bit_width {
            BitWidth::W32 => {
                let mut buffer: [u8; 0x28] = [0u8; 0x28];
                self.file.read_all(self.base_offset as u64 + 0x28*self.entry_position as u64, &mut buffer)?;
                Section::new(&buffer, self.bit_width, self.endianness)
            },
            BitWidth::W64 => {
                let mut buffer: [u8; 0x40] = [0u8; 0x40];
                self.file.read_all(self.base_offset as u64 + 0x40*self.entry_position as u64, &mut buffer)?;
                Section::new(&buffer, self.bit_width, self.endianness)
            },
        }
    }
}
impl<'a, RO: 'a+Volume> Iterator for SectionIterator<'a, RO> {
    type Item = Result<Section, ReturnCode>;
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
    pub fn new(data: &[u8], bit_width: BitWidth, endianness: Endianness) -> Result<Self, ReturnCode> {
        let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
            Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
            Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
        };
        match bit_width {
            BitWidth::W32 => {
                if data.len() != 0x28 {return Err(ReturnCode::IncorrectBufferLength)};
                Ok(Self {
                    name:            u32_fb(data[0x00..0x04].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    section_type: SectionType::try_from(
                                     u32_fb(data[0x04..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?))
                                                                       .map_err( |_| ReturnCode::InvalidData)?,
                    flags:           u32_fb(data[0x08..0x0C].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                    virtual_address: u32_fb(data[0x0C..0x10].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                    file_offset:     u32_fb(data[0x10..0x14].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                    file_size:       u32_fb(data[0x14..0x18].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                    link:            u32_fb(data[0x18..0x1C].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    info:            u32_fb(data[0x1C..0x20].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    alignment:       u32_fb(data[0x20..0x24].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                    entry_size:      u32_fb(data[0x24..0x28].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                })
            },
            BitWidth::W64 => {
                if data.len() != 0x40 {return Err(ReturnCode::IncorrectBufferLength)};
                Ok(Self {
                    name:            u32_fb(data[0x00..0x04].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    section_type: SectionType::try_from(
                                     u32_fb(data[0x04..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?))
                                                                       .map_err( |_| ReturnCode::InvalidData)?,
                    flags:           u64_fb(data[0x08..0x10].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    virtual_address: u64_fb(data[0x10..0x18].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    file_offset:     u64_fb(data[0x18..0x20].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    file_size:       u64_fb(data[0x20..0x28].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    link:            u32_fb(data[0x28..0x2C].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    info:            u32_fb(data[0x2C..0x30].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    alignment:       u64_fb(data[0x30..0x38].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    entry_size:      u64_fb(data[0x38..0x40].try_into().map_err( |_| ReturnCode::SlicingError)?),
                })
            },
        }
    }
}

//Section Type
numeric_enum! {
    #[repr(u32)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum SectionType {
        Null                           = 0x00000000,
        ProgramData                    = 0x00000001,
        SymbolTable                    = 0x00000002,
        StringTable                    = 0x00000003,
        ExplicitRelocationTable        = 0x00000004,
        SymbolHashTable                = 0x00000005,
        DynamicLinkingInformation      = 0x00000006,
        Notes                          = 0x00000007,
        ProgramNoData                  = 0x00000008,
        ImplicitRelocationTable        = 0x00000009,
        DynamicSymbolTable             = 0x0000000B,
        /*Reserved*/                                
        InitializationFunctionArray    = 0x0000000E,
        TerminationFunctionArray       = 0x0000000F,
        PreInitializationFunctionArray = 0x00000010,
        SectionGroup                   = 0x00000011,
        ExtendedSectionIndex           = 0x00000012,
    }
}

//Section Flags
numeric_enum! {
    #[repr(u64)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum SectionFlags {
        Writeable          = 0x0000000000000001,
        Allocated          = 0x0000000000000002,
        Executable         = 0x0000000000000004,
        /*Reserved*/                            
        Merged             = 0x0000000000000010,
        Strings            = 0x0000000000000020,
        InfoLink           = 0x0000000000000040,
        LinkOrder          = 0x0000000000000080,
        OSNonConforming    = 0x0000000000000100,
        GroupMember        = 0x0000000000000200,
        ThreadLocalStorage = 0x0000000000000400,
        Compressed         = 0x0000000000000800,
    }
}


// PROGRAM: DYNAMIC ENTRY
//Dynamic Entry
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct ProgramDynamicEntry {
    pub entry_type: ProgramDynamicEntryType,
    pub value:      u64,
}
impl ProgramDynamicEntry {
    pub fn new(data: &[u8], bit_width: BitWidth, endianness: Endianness) -> Result<Self, ReturnCode> {
        let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
            Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
            Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
        };
        match bit_width {
            BitWidth::W32 => {
                if data.len() != 8 {return Err(ReturnCode::IncorrectBufferLength)};
                Ok(Self {
                    entry_type: ProgramDynamicEntryType::try_from(
                           u32_fb(data[0x00..0x04].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64)
                                                             .map_err( |_| ReturnCode::InvalidData)?,
                    value: u32_fb(data[0x04..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                })
            },
            BitWidth::W64 => {
                if data.len() != 16 {return Err(ReturnCode::IncorrectBufferLength)};
                Ok(Self {
                    entry_type: ProgramDynamicEntryType::try_from(
                           u64_fb(data[0x00..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64)
                                                             .map_err( |_| ReturnCode::InvalidData)?,
                    value: u64_fb(data[0x08..0x10].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                })
            },
        }
    }
}

//Dynamic Table Iterator
pub struct ProgramDynamicEntryIterator<'a, RO: 'a+Volume> {
    file:       &'a RO,
    bit_width:      BitWidth,
    endianness:     Endianness,
    base_offset:    u64,
    entry_position: u64,
    entry_count:    u64,
}
impl<'a, RO: 'a+Volume> ProgramDynamicEntryIterator<'a, RO> {
    // FUNCTIONS
    //Constructor
    pub fn new(file: &'a RO, file_header: &Header, program_header: &Program) -> Self{
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
    fn entry(&mut self) -> Result<ProgramDynamicEntry, ReturnCode> {
        match self.bit_width {
            BitWidth::W32 => {
                let mut buffer: [u8; 8] = [0u8; 8];
                self.file.read_all(self.base_offset as u64 + 8*self.entry_position as u64, &mut buffer)?;
                ProgramDynamicEntry::new(&buffer, self.bit_width, self.endianness)
            },
            BitWidth::W64 => {
                let mut buffer: [u8; 16] = [0u8; 16];
                self.file.read_all(self.base_offset as u64 + 16*self.entry_position as u64, &mut buffer)?;
                ProgramDynamicEntry::new(&buffer, self.bit_width, self.endianness)
            },
        }
    }
}
impl<'a, RO: 'a+Volume> Iterator for ProgramDynamicEntryIterator<'a, RO> {
    type Item = Result<ProgramDynamicEntry, ReturnCode>;
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


// RELOCATION TABLE
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
    pub fn new(data: &[u8], bit_width: BitWidth, endianness: Endianness, relocation_type: RelocationType) -> Result<Self, ReturnCode> {
        let (_u16_fb, u32_fb, u64_fb): (fn([u8;2]) -> u16, fn([u8;4]) -> u32, fn([u8;8]) -> u64) = match endianness {
            Endianness::Little => (u16::from_le_bytes, u32::from_le_bytes, u64::from_le_bytes),
            Endianness::Big    => (u16::from_be_bytes, u32::from_be_bytes, u64::from_be_bytes),
        };
        match (bit_width, relocation_type) {
            (BitWidth::W32, RelocationType::Implicit) => {
                if data.len() != 0x08 {return Err(ReturnCode::IncorrectBufferLength);}
                let info: u32 = u32_fb(data[0x04..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?);
                Ok(Self {
                    offset: u32_fb(data[0x00..0x04].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                    symbol: info >> 8,
                    relocation_entry_type: RelocationEntryTypeX86_64::try_from(info & 0xFF).map_err( |_| ReturnCode::InvalidData)?,
                    addend: None,
                })
            },
            (BitWidth::W32, RelocationType::Explicit) => {
                if data.len() != 0x0C {return Err(ReturnCode::IncorrectBufferLength);}
                let info: u32 = u32_fb(data[0x04..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?);
                Ok(Self {
                    offset: u32_fb(data[0x00..0x04].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64,
                    symbol: info >> 8,
                    relocation_entry_type: RelocationEntryTypeX86_64::try_from(info & 0xFF).map_err( |_| ReturnCode::InvalidData)?,
                    addend: Some(u32_fb(data[0x08..0x0C].try_into().map_err( |_| ReturnCode::SlicingError)?) as u64),
                })
            },
            (BitWidth::W64, RelocationType::Implicit) => {
                if data.len() != 0x10 {return Err(ReturnCode::IncorrectBufferLength);}
                let info: u64 = u64_fb(data[0x08..0x10].try_into().map_err( |_| ReturnCode::SlicingError)?);
                Ok(Self {
                    offset: u64_fb(data[0x00..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    symbol: (info>>32) as u32,
                    relocation_entry_type: RelocationEntryTypeX86_64::try_from((info & 0xFFFFFFFF) as u32).map_err( |_| ReturnCode::InvalidData)?,
                    addend: None,
                })
            },
            (BitWidth::W64, RelocationType::Explicit) => {
                if data.len() != 0x18 {return Err(ReturnCode::IncorrectBufferLength);}
                let info: u64 = u64_fb(data[0x08..0x10].try_into().map_err( |_| ReturnCode::SlicingError)?);
                Ok(Self {
                    offset: u64_fb(data[0x00..0x08].try_into().map_err( |_| ReturnCode::SlicingError)?),
                    symbol: (info>>32) as u32,
                    relocation_entry_type: RelocationEntryTypeX86_64::try_from((info & 0xFFFFFFFF) as u32).map_err( |_| ReturnCode::InvalidData)?,
                    addend: Some(u64_fb(data[0x10..0x18].try_into().map_err( |_| ReturnCode::SlicingError)?)),
                })
            },
        }
    }
}

//Relocation Entry Iterator
pub struct RelocationEntryIterator <'a, RO: 'a+Volume> {
    file:  &'a       RO,
    bit_width:       BitWidth,
    endianness:      Endianness,
    relocation_type: RelocationType,
    file_offset:     u64,
    entry_position:  u64,
    entry_count:     u64,
}
impl<'a, RO: 'a+Volume> RelocationEntryIterator<'a, RO> {
    // CONSTRUCTOR
    pub fn new(file: &'a RO, bit_width: BitWidth, endianness: Endianness, relocation_type: RelocationType, file_size: u64, file_offset: u64) -> Self {
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
    pub fn entry(&mut self) -> Result<RelocationEntry, ReturnCode> {
        match (self.bit_width, self.relocation_type) {
            (BitWidth::W32, RelocationType::Implicit) => {
                let mut buffer: [u8; 0x08] = [0u8; 0x08];
                self.file.read_all(self.file_offset as u64 + 0x08*self.entry_position as u64, &mut buffer)?;
                RelocationEntry::new(&buffer, self.bit_width, self.endianness, self.relocation_type)
            },
            (BitWidth::W32, RelocationType::Explicit) => {
                let mut buffer: [u8; 0x0C] = [0u8; 0x0C];
                self.file.read_all(self.file_offset as u64 + 0x0C*self.entry_position as u64, &mut buffer)?;
                RelocationEntry::new(&buffer, self.bit_width, self.endianness, self.relocation_type)
            }, 
            (BitWidth::W64, RelocationType::Implicit) => {
                let mut buffer: [u8; 0x10] = [0u8; 0x10];
                self.file.read_all(self.file_offset as u64 + 0x10*self.entry_position as u64, &mut buffer)?;
                RelocationEntry::new(&buffer, self.bit_width, self.endianness, self.relocation_type)
            }, 
            (BitWidth::W64, RelocationType::Explicit) => {
                let mut buffer: [u8; 0x18] = [0u8; 0x18];
                self.file.read_all(self.file_offset as u64 + 0x18*self.entry_position as u64, &mut buffer)?;
                RelocationEntry::new(&buffer, self.bit_width, self.endianness, self.relocation_type)
            },
        }
    }
}
impl<'a, RO: 'a+Volume> Iterator for RelocationEntryIterator<'a, RO> {
    type Item = Result<RelocationEntry, ReturnCode>;
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


// CROSS-USE ENUMS
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
numeric_enum! {
    #[repr(u64)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum RelocationType {
        Explicit = 0x07,
        Implicit = 0x11,
    }
}
