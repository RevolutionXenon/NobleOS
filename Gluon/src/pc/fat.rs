// GLUON: PC FILE ALLOCATION TABLE
// Structs and enums related to the contents and handling of the FAT16 file system


// HEADER
//Flags
#![allow(clippy::needless_range_loop)]

//Imports
use crate::{numeric_enum, return_if_partial};
use crate::noble::file_system::*;
use crate::noble::return_code::ReturnCode;
use core::{convert::{TryFrom, TryInto}};
use core::str;


// FAT16 FILE SYSTEMS
//Full FAT16 Handling Routines
pub struct FATFileSystem<'s> {
    pub volume:     &'s dyn Volume,
    pub boot_sector:    FATBootSector,
    pub fat:            FATTable<'s>,
}
impl<'s>                FATFileSystem<'s> {
    // CONSTRUCTOR
    pub fn format_new(volume: &'s dyn Volume, boot_sector: FATBootSector) -> Result<Self, ReturnCode> {
        //Clear
        for i in 0..(boot_sector.data_start_sector() * boot_sector.bytes_per_sector as u32) as u64 / 0x200 {  
            volume.write_all(0x200*i, &[0u8; 0x200])?;
        }
        //Write Boot Sector
        volume.write_all(0, &<[u8;0x200]>::try_from(boot_sector)?)?;
        //Create FAT
        let fat = FATTable::new(volume, boot_sector);
        fat.write_raw(0, 0xFFF0)?;
        fat.write_raw(1, 0xFFFF)?;
        //Return
        Ok(Self {volume, boot_sector, fat})
    }
    pub fn from_existing_volume(volume: &'s dyn Volume) -> Result<Self, ReturnCode> {
        //Load Boot Sector
        let mut buffer: [u8; 0x200] = [0u8; 0x200];
        volume.read_all(0x00, &mut buffer)?;
        let boot_sector = FATBootSector::try_from(buffer)?;
        //Load FAT
        let fat = FATTable::new(volume, boot_sector);
        //Return
        Ok(Self {volume, boot_sector, fat})
    }
}
impl<'s> FileSystem for FATFileSystem<'s> {
    //Read and write files
    fn read        (&self, id: OpenFileID, offset: u64, buffer: &mut [u8])             -> Result<u64,         ReturnCode> {
        if id.0 == 0 {
            let file = VolumeFromVolume {
                volume: self.volume,
                offset: self.boot_sector.root_location() as u64,
                size: (self.boot_sector.root_entry_count as u64 * 32) as u64,
            };
            file.read(offset, buffer)
        }
        else {
            let file = FATFile::new_from_start_cluster(self, id.0 as u32)?;
            file.read(offset, buffer)
        }
    }
    fn write       (&self, id: OpenFileID, offset: u64, buffer: &[u8])                 -> Result<u64,         ReturnCode> {
        if id.0 == 0 {
            let file = VolumeFromVolume {
                volume: self.volume,
                offset: self.boot_sector.root_location() as u64,
                size: self.boot_sector.root_size() as u64,
            };
            file.write(offset, buffer)
        }
        else {
            let file = FATFile::new_from_start_cluster(self, id.0 as u32)?;
            file.write(offset, buffer)
        }
    }
    //Open and close files
    fn open        (&self, id: FileID)                                                 -> Result<OpenFileID,  ReturnCode> {
        if id.0 == 1 || id.0 >= self.boot_sector.fat_entry_count() as u64 {return Err(ReturnCode::InvalidIdentifier)}
        Ok(OpenFileID(id.0))
    }
    fn close       (&self, _id: OpenFileID)                                            -> Result<(),          ReturnCode> {
        Ok(())
    }
    //Create and delete files
    fn create      (&self, directory_id: OpenFileID, name: &str, size: u64, dir: bool) -> Result<OpenFileID,  ReturnCode> {
        //Checks
        let file_size: u32 = u32::try_from(size).map_err(|_| ReturnCode::VolumeOutOfBounds)?;
        //Objects
        let directory: FATDirectory;
        let root;
        let file;
        //Root Directory
        if directory_id.0 == 0 {
            root = VolumeFromVolume {
                volume: self.volume,
                offset: self.boot_sector.root_location() as u64,
                size: self.boot_sector.root_entry_count as u64 * 32,
            };
            directory = FATDirectory{ directory: &root};
        }
        //Other Directory
        else {
            file = FATFile::new_from_start_cluster(self, directory_id.0 as u32)?;
            directory = FATDirectory{directory: &file};
        }
        //Find free entry
        let entry_position = directory.find_free_entry()?;
        //Allocate
        let start_cluster = self.fat.allocate_clusters((file_size / self.boot_sector.cluster_size()) as u16)?;
        //Create entry data
        let directory_entry = FATShortDirectoryEntry {
            file_name: if dir {format_short_directory_name(name)?} else {format_short_file_name(name)?},
            file_attributes: FATFileAttributes {
                read_only:       false,
                hidden:          false,
                system:          false,
                volume_label:    false,
                subdirectory:    dir,
                archive:         false,
                reserved_6:      false,
                reserved_7:      false,
            },
            start_cluster_high: (start_cluster >> 16) as u16,
            start_cluster_low: (start_cluster & 0xFFFF) as u16,
            creation_time_ss: 0,
            creation_time: 0,
            creation_date: 0,
            file_size,
        };
        //Write to directory
        directory.write_entry(entry_position, directory_entry)?;
        //Finish
        Ok(OpenFileID(start_cluster as u64))
    }
    fn delete      (&self, directory_id: OpenFileID, name: &str)                       -> Result<(),          ReturnCode> {
        todo!()
    }
    //Traverse directories
    fn root        (&self)                                                             -> Result<FileID,      ReturnCode> {
        Ok(FileID(0))
    }
    fn dir_first   (&self, directory_id: OpenFileID)                                   -> Result<Option<u64>, ReturnCode> {
        let directory = FATDirectory{directory: &FileShortcut{fs: self, id: directory_id}};
        directory.find_shortname_entry(0).map(|o| o.map(|i| i as u64))
    }
    fn dir_next    (&self, directory_id: OpenFileID, index: u64)                       -> Result<Option<u64>, ReturnCode> {
        let directory = FATDirectory{directory: &FileShortcut{fs: self, id: directory_id}};
        directory.find_shortname_entry((index + 1) as u32).map(|o| o.map(|i| i as u64))
    }
    fn dir_name    (&self, directory_id: OpenFileID, name: &str)                       -> Result<Option<u64>, ReturnCode> {
        let mut index = match self.dir_first(directory_id) {
            Ok(Some(index)) => index,
            finish => return finish,
        };
        loop {
            let mut buffer = [0u8;12];
            let entry_name = self.get_name(directory_id, index, &mut buffer)?;
            if entry_name == name {return Ok(Some(index))}
            index = match self.dir_next(directory_id, index) {
                Ok(Some(index)) => index,
                finish => return finish,
            };
        }
    }
    //File properties
    fn get_id      (&self, directory_id: OpenFileID, index: u64)                       -> Result<FileID,      ReturnCode> {
        let directory = FATDirectory{directory: &FileShortcut{fs: self, id: directory_id}};
        let entry = directory.read_entry(index as u32)?;
        if !entry.query_shortname() {return Err(ReturnCode::InvalidIdentifier)};
        Ok(FileID(entry.start_cluster_32() as u64))
    }
    fn get_dir     (&self, directory_id: OpenFileID, index: u64)                       -> Result<bool,        ReturnCode> {
        let directory = FATDirectory{directory: &FileShortcut{fs: self, id: directory_id}};
        let entry = directory.read_entry(index as u32)?;
        if !entry.query_shortname() {return Err(ReturnCode::InvalidIdentifier)};
        Ok(entry.file_attributes.subdirectory)
    }
    fn get_size    (&self, directory_id: OpenFileID, index: u64)                       -> Result<u64,         ReturnCode> {
        let directory = FATDirectory{directory: &FileShortcut{fs: self, id: directory_id}};
        let entry = directory.read_entry(index as u32)?;
        if !entry.query_shortname() {return Err(ReturnCode::InvalidIdentifier)};
        Ok(entry.file_size as u64)
    }
    fn set_size    (&self, directory_id: OpenFileID, index: u64, size: u64)            -> Result<(),          ReturnCode> {
        todo!()
    }
    fn get_name<'f>(&self, directory_id: OpenFileID, index: u64, buffer: &'f mut[u8])  -> Result<&'f str,     ReturnCode> {
        let directory = FATDirectory{directory: &FileShortcut{fs: self, id: directory_id}};
        let entry = directory.read_entry(index as u32)?;
        if !entry.query_shortname() {return Err(ReturnCode::InvalidIdentifier)};
        match entry.file_attributes.subdirectory {
            true => retrieve_short_directory_name(entry.file_name, buffer),
            false => retrieve_short_file_name(entry.file_name, buffer),
        }
    }
    fn set_name    (&self, directory_id: OpenFileID, index: u64, name: &str)           -> Result<(),          ReturnCode> {
        todo!()
    }
}


// NAME CONVERSION
fn format_short_file_name       (name_in: &str) -> Result<[u8;11], ReturnCode> {
    //Check name string is valid
    if !name_in.is_ascii() {return Err(ReturnCode::InvalidCharacter)}
    let name_bytes = name_in.as_bytes();
    if name_bytes.len() > 12 {return Err(ReturnCode::DataTooLarge)}
    //Find delimiter between name and extension
    let mut delimiter_index: Option<usize> = None;
    for i in 0..name_in.len() {
        if name_bytes[i] == 0x2E {delimiter_index = Some(i)}
    }
    //Determine name and extension from array
    let name_array = match delimiter_index {
        Some(i) => &name_bytes[0..i],
        None => name_bytes,
    };
    let ext_array = match delimiter_index {
        Some(i) => &name_bytes[i+1..],
        None => &[],
    };
    //Check name and extension are valid
    if name_array.len() > 8 {return Err(ReturnCode::DataTooLarge)}
    if ext_array.len()  > 3 {return Err(ReturnCode::DataTooLarge)}
    //Create name array
    let mut name_final = [0x20u8; 11];
    name_final[0..name_array.len()].clone_from_slice(name_array);
    name_final[8..8+ext_array.len()].clone_from_slice(ext_array);
    //Finish
    Ok(name_final)
}
fn format_short_directory_name  (name_in: &str) -> Result<[u8;11], ReturnCode> {
    //Check name string is valid
    if !name_in.is_ascii() {return Err(ReturnCode::InvalidCharacter)}
    let input_bytes = name_in.as_bytes();
    if input_bytes.len() > 11 {return Err(ReturnCode::DataTooLarge)}
    //Create name array
    let mut output_bytes = [0x20u8;11];
    output_bytes[..input_bytes.len()].copy_from_slice(input_bytes);
    //Finish
    Ok(output_bytes)

}
fn retrieve_short_file_name     (ascii_in: [u8;11], buffer: &mut[u8]) -> Result<&str, ReturnCode> {
    if buffer.len() < 12 {return Err(ReturnCode::BufferTooSmall)}
    //Get File Name
    let mut buffer_index = 0;
    for i in 0..8 {
        let ascii_byte = ascii_in[i];
        if ascii_byte == 0x20 {break}
        buffer[buffer_index] = ascii_byte;
        buffer_index += 1;
    }
    //Check for extension
    if ascii_in[8] != 0x20 {
        //Add delimiter
        buffer[buffer_index] = 0x2E;
        buffer_index += 1;
        //Get extension
        for i in 8..11 {
            let ascii_byte = ascii_in[i];
            if ascii_byte == 0x20 {break}
            buffer[buffer_index] = ascii_byte;
            buffer_index += 1;
        }
    }
    //Finish
    let ret_str = str::from_utf8(&buffer[0..buffer_index]).map_err(|_| ReturnCode::ConversionError)?;
    Ok(ret_str)
}
fn retrieve_short_directory_name(ascii_in: [u8;11], buffer: &mut[u8]) -> Result<&str, ReturnCode> {
    if buffer.len() < 11 {return Err(ReturnCode::BufferTooSmall)}
    //Get Name
    let mut buffer_index = 0;
    for i in 0..11 {
        let ascii_byte = ascii_in[i];
        if ascii_byte == 0x20 {break}
        buffer[buffer_index] = ascii_byte;
        buffer_index += 1;
    }
    //Finish
    let ret_str = str::from_utf8(&buffer[0..buffer_index]).map_err(|_| ReturnCode::ConversionError)?;
    Ok(ret_str)
}


// FAT16 BOOT SECTOR
//Boot Sector
#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct FATBootSector {
    pub jump_instruction:     [ u8; 0x0003], //Jump instruction
    pub oem_name:             [ u8; 0x0008], //Name string, usually "MSWIN4.1"
    pub bytes_per_sector:      BytesPerSector,      //512, 1024, 2048, or 4096 only
    pub sectors_per_cluster:   SectorsPerCluster,   //Power of 2 greater than 2^0, should not result in a bytes per cluster > 32KB
    pub reserved_sector_count: u16,
    pub fat_number:             u8,     //Usually 2
    pub root_entry_count:      u16,     //Should be aligned so root_entry_count * 32 / bytes_per_sector 
    pub total_sectors_16:      u16,
    pub media:                 MediaType,
    pub sectors_per_fat:       u16,
    pub sectors_per_track:     u16,
    pub heads_number:          u16,
    pub hidden_sectors:        u32,
    pub total_sectors_32:      u32,
    pub drive_number:           u8,
    pub volume_id:             u32,
    pub volume_label:         [ u8; 0x000B],
    pub file_system_type:     [ u8; 0x0008],
    pub bootstrap_code:       [ u8; 0x01C0],
}
impl FATBootSector {
    pub fn total_sectors     (&self) -> u32 {if self.total_sectors_16 != 0 {self.total_sectors_16 as u32} else {self.total_sectors_32}}
    pub fn cluster_size      (&self) -> u32 {self.bytes_per_sector as u32 * self.sectors_per_cluster as u32}
    pub fn fat_start_sector  (&self) -> u32 {self.reserved_sector_count as u32}
    pub fn fat_location      (&self) -> u32 {self.fat_start_sector() * self.bytes_per_sector as u32}
    pub fn fat_size          (&self) -> u32 {self.sectors_per_fat as u32 * self.bytes_per_sector as u32}
    pub fn fat_entry_count   (&self) -> u32 {self.fat_size() / 2}
    pub fn root_start_sector (&self) -> u32 {self.fat_start_sector() + (self.sectors_per_fat as u32 * self.fat_number as u32)}
    pub fn root_location     (&self) -> u32 {self.root_start_sector() * self.bytes_per_sector as u32}
    pub fn root_size         (&self) -> u32 {self.root_entry_count as u32 * 32}
    pub fn data_start_sector (&self) -> u32 {self.root_start_sector() + ((self.root_entry_count as u32 * 32) / self.bytes_per_sector as u32)}
    pub fn data_location     (&self) -> u32 {self.data_start_sector() * self.bytes_per_sector as u32}
}
impl TryFrom<[u8;0x200]> for FATBootSector {
    type Error = ReturnCode;

    fn try_from(bytes: [u8;512]) -> Result<Self, Self::Error> {
        if bytes[0x0026]        != 0x29         {return Err(ReturnCode::InvalidData)}
        if bytes[0x01FE..0x200] != [0x55, 0xAA] {return Err(ReturnCode::InvalidIdentifier)}
        Ok( Self {
            jump_instruction:                                                       bytes[0x0000..0x0003].try_into().map_err(|_| ReturnCode::SlicingError)?,
            oem_name:                                                               bytes[0x0003..0x000B].try_into().map_err(|_| ReturnCode::SlicingError)?,
            bytes_per_sector:           BytesPerSector::try_from(u16::from_le_bytes(bytes[0x000B..0x000D].try_into().map_err(|_| ReturnCode::SlicingError)?))
                                                                                                                    .map_err(|_| ReturnCode::InvalidData)?,
            sectors_per_cluster:     SectorsPerCluster::try_from(                   bytes[0x000D])                  .map_err(|_| ReturnCode::InvalidData)?,
            reserved_sector_count:                               u16::from_le_bytes(bytes[0x000E..0x0010].try_into().map_err(|_| ReturnCode::SlicingError)?),
            fat_number:                                                             bytes[0x0010],
            root_entry_count:                                    u16::from_le_bytes(bytes[0x0011..0x0013].try_into().map_err(|_| ReturnCode::SlicingError)?),
            total_sectors_16:                                    u16::from_le_bytes(bytes[0x0013..0x0015].try_into().map_err(|_| ReturnCode::SlicingError)?),
            media:                           MediaType::try_from(                   bytes[0x0015]                  ).map_err(|_| ReturnCode::InvalidData)?,
            sectors_per_fat:                                     u16::from_le_bytes(bytes[0x0016..0x0018].try_into().map_err(|_| ReturnCode::SlicingError)?),
            sectors_per_track:                                   u16::from_le_bytes(bytes[0x0018..0x001A].try_into().map_err(|_| ReturnCode::SlicingError)?),
            heads_number:                                        u16::from_le_bytes(bytes[0x001A..0x001C].try_into().map_err(|_| ReturnCode::SlicingError)?),
            hidden_sectors:                                      u32::from_le_bytes(bytes[0x001C..0x0020].try_into().map_err(|_| ReturnCode::SlicingError)?),
            total_sectors_32:                                    u32::from_le_bytes(bytes[0x0020..0x0024].try_into().map_err(|_| ReturnCode::SlicingError)?),
            drive_number:                                                           bytes[0x0024],
            volume_id:                                           u32::from_le_bytes(bytes[0x0027..0x002B].try_into().map_err(|_| ReturnCode::SlicingError)?),
            volume_label:                                                           bytes[0x002B..0x0036].try_into().map_err(|_| ReturnCode::SlicingError)?,
            file_system_type:                                                       bytes[0x0036..0x003E].try_into().map_err(|_| ReturnCode::SlicingError)?,
            bootstrap_code:                                                         bytes[0x003E..0x01FE].try_into().map_err(|_| ReturnCode::SlicingError)?,
        })
    }
}
impl TryFrom<FATBootSector> for [u8;0x200] {
    type Error = ReturnCode;

    fn try_from(sector: FATBootSector) -> Result<Self, Self::Error> {
        let mut bytes = [0u8;0x200];
        bytes[0x0000..0x0003].clone_from_slice(&sector.jump_instruction);
        bytes[0x0003..0x000B].clone_from_slice(&sector.oem_name);
        bytes[0x000B..0x000D].clone_from_slice(&(sector.bytes_per_sector as u16).to_le_bytes());
        bytes[0x000D] = sector.sectors_per_cluster as u8;
        bytes[0x000E..0x0010].clone_from_slice(&(sector.reserved_sector_count as u16).to_le_bytes());
        bytes[0x0010] = sector.fat_number;
        bytes[0x0011..0x0013].clone_from_slice(&sector.root_entry_count.to_le_bytes());
        bytes[0x0013..0x0015].clone_from_slice(&sector.total_sectors_16.to_le_bytes());
        bytes[0x0015] = sector.media as u8;
        bytes[0x0016..0x0018].clone_from_slice(&sector.sectors_per_fat.to_le_bytes());
        bytes[0x0018..0x001A].clone_from_slice(&sector.sectors_per_track.to_le_bytes());
        bytes[0x001A..0x001C].clone_from_slice(&sector.heads_number.to_le_bytes());
        bytes[0x001C..0x0020].clone_from_slice(&sector.hidden_sectors.to_le_bytes());
        bytes[0x0020..0x0024].clone_from_slice(&sector.total_sectors_32.to_le_bytes());
        bytes[0x0024] = sector.drive_number;
        bytes[0x0025] = 0x00;
        bytes[0x0026] = 0x29;
        bytes[0x0027..0x002B].clone_from_slice(&sector.volume_id.to_le_bytes());
        bytes[0x002B..0x0036].clone_from_slice(&sector.volume_label);
        bytes[0x0036..0x003E].clone_from_slice(&sector.file_system_type);
        bytes[0x003E..0x01FE].clone_from_slice(&sector.bootstrap_code);
        bytes[0x01FE] = 0x55;
        bytes[0x01FF] = 0xAA;
        Ok(bytes)
    }
}

//Bytes Per Sector
numeric_enum! {
    #[repr(u16)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum BytesPerSector {
        bps_512  = 0x0200,
        bps_1024 = 0x0400,
        bps_2048 = 0x0800,
        bps_4096 = 0x1000,
    }
}

//Sectors Per Cluster
numeric_enum! {
    #[repr(u8)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum SectorsPerCluster {
        spc_1  = 0x01,
        spc_2  = 0x02,
        spc_4  = 0x04,
        spc_8  = 0x08,
        spc_16 = 0x10,
        spc_32 = 0x20,
        spc_64 = 0x40,
    }
}

//Media Type
numeric_enum! {
    #[repr(u8)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum MediaType {
        FLOPPY_250   = 0xE5,
        FLOPPY_720   = 0xED,
        NON_STANDARD = 0xEE,
        SUPERFLOPPY  = 0xEF,
        FLOPPY_1440  = 0xF0,
        ALTOS_DD     = 0xF4,
        ALTOS_FD     = 0xF5,
        FIXED_DISK   = 0xF8,
        FLOPPY       = 0xF9,
        FLOPPY_320   = 0xFA,
        FLOPPY_640   = 0xFB,
        FLOPPY_180   = 0xFC,
        FLOPPY_360   = 0xFD,
        FLOPPY_160   = 0xFE,
        FLOPPY_5_320 = 0xFF,
    }
}


// FAT STRUCTURE
//File Allocation Table
pub struct FATTable<'s> {
    volume: &'s dyn Volume,
    start_location: u32,
    entry_count:    u32,
    fat_count:      u32,
    fat_size:       u32,
}
impl<'s> FATTable<'s> {
    // CONSTRUCTOR
    pub fn new(volume: &'s dyn Volume, boot_sector: FATBootSector) -> Self {
        Self {
            volume,
            start_location: boot_sector.fat_location(),
            entry_count:    (boot_sector.total_sectors() - boot_sector.data_start_sector()) / boot_sector.sectors_per_cluster as u32 + 2,
            fat_count:      boot_sector.fat_number as u32,
            fat_size:       boot_sector.fat_size()
        }
    }
    // READ ONLY
    pub fn read_entry(&self, cluster: u16) -> Result<FATTableEntry, ReturnCode> {
        if cluster as u32 > self.entry_count {return Err(ReturnCode::IndexOutOfBounds)}
        let mut buffer = [0u8;2];
        let table_offset = (cluster * 2) as u32;
        self.volume.read_all((self.start_location + table_offset) as u64, &mut buffer)?;
        let entry_raw = u16::from_le_bytes(buffer);
        Ok(match entry_raw {
            0x0000          => FATTableEntry::Free,
            0x0001          => FATTableEntry::Reserved,
            0x0002..=0xFFEF => FATTableEntry::Used(entry_raw),
            0xFFF0..=0xFFF6 => FATTableEntry::Reserved,
            0xFFF7          => FATTableEntry::Bad,
            0xFFF8..=0xFFFF => FATTableEntry::End,
        })
    }
    pub fn find_free(&self, start_cluster: u32) -> Result<u32, ReturnCode> {
        for i in start_cluster..self.entry_count {
            if let FATTableEntry::Free = self.read_entry(i as u16)? { return Ok(i) }
        }
        Err(ReturnCode::VolumeFull)
    }
    // WRITE ONLY
    pub fn write_entry(&self, cluster: u16, entry: FATTableEntry) -> Result<(), ReturnCode> {
        if cluster as u32 > self.entry_count {return Err(ReturnCode::IndexOutOfBounds)}
        let table_offset = (cluster * 2) as u32;
        let entry_bytes = match entry {
            FATTableEntry::Free        => 0x0000,
            FATTableEntry::Reserved    => 0x0001,
            FATTableEntry::Used(v) => v,
            FATTableEntry::Bad         => 0xFFF7,
            FATTableEntry::End         => 0xFFFF,
        }.to_le_bytes();
        for i in 0..self.fat_count {
            self.volume.write_all((self.start_location + i*self.fat_size + table_offset) as u64, &entry_bytes)?;
        }
        Ok(())
    }
    pub fn write_raw  (&self, cluster: u16, entry: u16) -> Result<(), ReturnCode> {
        if cluster as u32 > self.entry_count {return Err(ReturnCode::IndexOutOfBounds)}
        let table_offset = (cluster * 2) as u32;
        let entry_bytes = entry.to_le_bytes();
        for i in 0..self.fat_count {
            self.volume.write_all((self.start_location + (i*self.fat_size) + table_offset) as u64, &entry_bytes)?;
        }
        Ok(())
    }
    // ALLOCATE
    pub fn allocate_clusters(&self, cluster_count: u16) -> Result<u32, ReturnCode> {
        let mut free_cluster = 2;
        for _ in 0..cluster_count {
            free_cluster = self.find_free(free_cluster)?;
        }
        let first_cluster = self.find_free(2)?;
        let mut previous_cluster = first_cluster;
        for _ in 1..cluster_count {
            let next_cluster = self.find_free(previous_cluster + 1)?;
            self.write_entry(previous_cluster as u16, FATTableEntry::Used(next_cluster as u16))?;
            previous_cluster = next_cluster;
        }
        self.write_entry(previous_cluster as u16, FATTableEntry::End)?;
        Ok(first_cluster)
    }
}

//File Allocation Table Entry
#[derive(Debug)]
pub enum FATTableEntry {
    Free,
    Reserved,
    Used(u16),
    Bad,
    End,
}


// FAT DIRECTORY
//Directory
pub struct FATDirectory<'s> {
    pub directory: &'s dyn Volume,
}
impl<'s> FATDirectory<'s> {
    // BASIC
    pub fn read_entry(&self, index: u32) -> Result<FATShortDirectoryEntry, ReturnCode> {
        //if position >= self.num_entries {return Err(ReturnCode::IndexOutOfBounds)}
        FATShortDirectoryEntry::try_from({
            let mut buffer = [0u8;0x20];
            self.directory.read_all(index as u64 * 32, &mut buffer)?;
            buffer
        })
    }
    pub fn write_entry(&self, index: u32, entry: FATShortDirectoryEntry) -> Result<(), ReturnCode> {
        let buffer: [u8; 0x20] = <[u8;0x20]>::try_from(entry)?;
        self.directory.write_all(index as u64 * 32, &buffer)?;
        Ok(())
    }
    // FIND
    pub fn find_shortname_entry(&self, mut index: u32) -> Result<Option<u32>, ReturnCode> {
        loop {
            let entry = match self.read_entry(index) {
                Ok(entry) => entry,
                Err(error) => match error {
                    ReturnCode::EndOfVolume => {return Ok(None)},
                    _ => {return Err(error)}
                },
            };
            if entry.query_shortname() {return Ok(Some(index))}
            index += 1;
        }
    }
    pub fn find_free_entry(&self) -> Result<u32, ReturnCode> {
        let mut index = 0;
        loop {
            let entry = self.read_entry(index)?;
            if entry.query_free() {return Ok(index)}
            index += 1;
        }
    }
}

//Directory Entry
#[derive(Debug)]
pub struct FATShortDirectoryEntry {
    pub file_name:          [u8; 11],          //Bytes 0x00 - 0x0A
    pub file_attributes:    FATFileAttributes, //Byte  0x0B
    pub creation_time_ss:   u8,                //Byte  0x0D
    pub creation_time:      u16,               //Bytes 0x0E - 0x0F
    pub creation_date:      u16,               //Bytes 0x10 - 0x11
    pub start_cluster_high: u16,               //Bytes 0x14 - 0x15
    pub start_cluster_low:  u16,               //Bytes 0x1A - 0x1B
    pub file_size:          u32,               //Bytes 0x1C - 0x1F
}
impl FATShortDirectoryEntry {
    // FIELDS
    pub fn start_cluster_16(&self) -> Result<u16, ReturnCode> {
        if self.start_cluster_high > 0 {return Err(ReturnCode::InvalidData)}
        Ok(self.start_cluster_low)
    }
    pub fn start_cluster_32(&self) -> u32 {
        ((self.start_cluster_high as u32) << 16) + self.start_cluster_low as u32
    }
    // QUERY
    pub fn query_deleted(&self) -> bool {
        self.file_name[0] == 0xE5
    }
    pub fn query_end(&self) -> bool {
        self.file_name[0] == 0x00
    }
    pub fn query_free(&self) -> bool {
        self.query_deleted() || self.query_end()
    }
    pub fn query_shortname(&self) -> bool {
        !self.file_attributes.query_longname() && !self.query_free()
    }
    pub fn query_longname(&self) -> bool {
        if self.file_name[0] == 0x00 || self.file_name[0] == 0xE5 {return false}
        self.file_attributes.query_longname() && !self.query_free()
    }
}
impl TryFrom<[u8;0x20]> for FATShortDirectoryEntry {
    type Error = ReturnCode;

    fn try_from(bytes: [u8;0x20]) -> Result<Self, Self::Error> {
        Ok(Self {
            file_name:          bytes[0x00..0x0B].try_into().map_err(|_| ReturnCode::SlicingError)?,
            file_attributes:    FATFileAttributes::from(TryInto::<u8>::try_into(bytes[0x0B]).map_err(|_| ReturnCode::ConversionError)?),
            creation_time_ss:   bytes[0x0D],
            start_cluster_high: u16::from_le_bytes(bytes[0x14..0x16].try_into().map_err(|_| ReturnCode::SlicingError)?),
            creation_time:      u16::from_le_bytes(bytes[0x16..0x18].try_into().map_err(|_| ReturnCode::SlicingError)?),
            creation_date:      u16::from_le_bytes(bytes[0x18..0x1A].try_into().map_err(|_| ReturnCode::SlicingError)?),
            start_cluster_low:  u16::from_le_bytes(bytes[0x1A..0x1C].try_into().map_err(|_| ReturnCode::SlicingError)?),
            file_size:          u32::from_le_bytes(bytes[0x1C..0x20].try_into().map_err(|_| ReturnCode::SlicingError)?),
        })
    }
}
impl TryFrom<FATShortDirectoryEntry> for [u8;0x20] {
    type Error = ReturnCode;

    fn try_from(entry: FATShortDirectoryEntry) -> Result<Self, Self::Error> {
        if entry.creation_time_ss >= 200 {return Err(ReturnCode::InvalidData)}
        let mut bytes = [0u8;0x20];
        bytes[0x00..0x0B].clone_from_slice(&entry.file_name);
        bytes[0x0B] = u8::from(entry.file_attributes);
        bytes[0x0D] = entry.creation_time_ss;
        bytes[0x14..0x16].clone_from_slice(&entry.start_cluster_high.to_le_bytes());
        bytes[0x16..0x18].clone_from_slice(&entry.creation_time.to_le_bytes());
        bytes[0x18..0x1A].clone_from_slice(&entry.creation_date.to_le_bytes());
        bytes[0x1A..0x1C].clone_from_slice(&entry.start_cluster_low.to_le_bytes());
        bytes[0x1C..0x20].clone_from_slice(&entry.file_size.to_le_bytes());
        Ok(bytes)
    }
}

//File Attributes
#[derive(Debug)]
pub struct FATFileAttributes {
    pub read_only:       bool, //Bit  0
    pub hidden:          bool, //Bit  1
    pub system:          bool, //Bit  2
    pub volume_label:    bool, //Bit  3
    pub subdirectory:    bool, //Bit  4
    pub archive:         bool, //Bit  5
    pub reserved_6:      bool, //Bit  6
    pub reserved_7:      bool, //Bit  7
}
impl FATFileAttributes {
    // CONSTRUCTOR
    pub fn new_file(read_only: bool, hidden: bool, system: bool) -> Self {
        Self {
            read_only,
            hidden,
            system,
            volume_label:    false,
            subdirectory:    false,
            archive:         false,
            reserved_6:      false,
            reserved_7:      false,
        }
    }
    pub fn new_directory(read_only: bool, hidden: bool, system: bool) -> Self {
        Self {
            read_only,
            hidden,
            system,
            volume_label:    false,
            subdirectory:    true,
            archive:         false,
            reserved_6:      false,
            reserved_7:      false,
        }
    }
    // QUERY
    pub fn query_longname(&self) -> bool {
        self.read_only && self.hidden && self.system && self.volume_label
    }
}
impl From<u8> for FATFileAttributes {
    fn from(byte: u8) -> Self {
        Self {
            read_only:       byte & 0b0000_0001 > 0,
            hidden:          byte & 0b0000_0010 > 0,
            system:          byte & 0b0000_0100 > 0,
            volume_label:    byte & 0b0000_1000 > 0,
            subdirectory:    byte & 0b0001_0000 > 0,
            archive:         byte & 0b0010_0000 > 0,
            reserved_6:      byte & 0b0100_0000 > 0,
            reserved_7:      byte & 0b1000_0000 > 0,
        }
    }
}
impl From<FATFileAttributes> for u8 {
    fn from(attributes: FATFileAttributes) -> Self {
        (if attributes.read_only       {0b0000_0001} else {0} +
         if attributes.hidden          {0b0000_0010} else {0} +
         if attributes.system          {0b0000_0100} else {0} +
         if attributes.volume_label    {0b0000_1000} else {0} +
         if attributes.subdirectory    {0b0001_0000} else {0} +
         if attributes.archive         {0b0010_0000} else {0} +
         if attributes.reserved_6      {0b0100_0000} else {0} +
         if attributes.reserved_7      {0b1000_0000} else {0})
    }
}


// FILE HANDLE
//FAT File
pub struct FATFile<'s> {
    volume:              &'s dyn Volume,
    fat:                 &'s FATTable<'s>,
    data_area_offset:        u32,
    start_cluster:           u32,
    cluster_size:            u32,
}
impl <'s>            FATFile<'s> {
    // CONSTRUCTOR
    pub fn new_from_start_cluster(fs: &'s FATFileSystem, start_cluster: u32) -> Result<Self, ReturnCode> {
        if start_cluster == 0 || start_cluster == 1 {return Err(ReturnCode::InvalidIdentifier)}
        let bs = fs.boot_sector;
        let fat = &fs.fat;
        Ok(Self {
            volume: fs.volume,
            fat,
            data_area_offset: bs.data_start_sector() * bs.bytes_per_sector as u32,
            start_cluster,
            cluster_size: bs.cluster_size(),
        })
    }
}
impl <'s> Volume for FATFile<'s> {
    // READ ONLY
    fn read (&self, offset: u64, buffer: &mut [u8]) -> Result<u64, ReturnCode> {
        //Test bounds to ensure read won't fail
        if buffer.is_empty()                       {return Ok(0)}
        let read_offset: u32 = u32::try_from(offset      ).map_err(|_| ReturnCode::VolumeOutOfBounds)?;
        let buffer_len:  u32 = u32::try_from(buffer.len()).map_err(|_| ReturnCode::BufferTooLarge)?;
        //if read_offset+buffer_len > self.file_size {return Err(ReturnCode::VolumeOutOfBounds)}
        //Calculate cluster and byte offsets into volume
        let start_cluster_offset: u32 = read_offset % self.cluster_size;
        let start_index:          u32 = read_offset / self.cluster_size;
        let end_index:            u32 = (read_offset + buffer_len - 1) / self.cluster_size;
        let end_cluster_offset:   u32 = (read_offset + buffer_len - 1) % self.cluster_size + 1;
        //Move to first cluster which needs to be read from
        let mut current_cluster: u32 = self.start_cluster;
        for _ in 0..start_index {
            //Retrieve next cluster
            current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                FATTableEntry::Used(cluster) => cluster as u32,
                FATTableEntry::End => {return Err(ReturnCode::EndOfVolume)},
                _ => {return Err(ReturnCode::SeekError)}
            };
        }
        //Start reading
        let mut buffer_offset: usize = 0;
        for index in start_index..=end_index {
            //If only one cluster needs to be accessed
            if index == start_index && index == end_index {
                //Read Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size + start_cluster_offset;
                return self.volume.read(volume_offset as u64, buffer);
            }
            //If its in the first cluster (may need to truncate)
            else if index == start_index {
                //Read Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size + start_cluster_offset;
                let new_buffer_offset: usize = (self.cluster_size - start_cluster_offset) as usize;
                return_if_partial!(0, self.volume.read_check(volume_offset as u64, &mut buffer[0..new_buffer_offset]));
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    FATTableEntry::End => {return Ok(buffer_offset as u64)},
                    _ => {return Err(ReturnCode::SeekError)}
                };
            }
            //If its in the final cluster (may need to truncate)
            else if index == end_index {
                //Read Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size;
                let new_buffer_offset = buffer_offset+end_cluster_offset as usize;
                return_if_partial!(buffer_offset as u64, self.volume.read_check(volume_offset as u64, &mut buffer[buffer_offset..new_buffer_offset]));
                buffer_offset = new_buffer_offset;
            }
            //If its in the middle (always reads entire cluster)
            else {
                //Read Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size;
                let new_buffer_offset: usize = buffer_offset + self.cluster_size as usize;
                return_if_partial!(buffer_offset as u64, self.volume.read_check(volume_offset as u64, &mut buffer[buffer_offset..new_buffer_offset]));
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    FATTableEntry::End => {return Ok(buffer_offset as u64)},
                    _ => {return Err(ReturnCode::SeekError)}
                };
            }
        }
        Ok(buffer_offset as u64)
    }
    // WRITE
    fn write(&self, offset: u64, buffer: &[u8])     -> Result<u64, ReturnCode> {
        //Test bounds to ensure read won't fail
        if buffer.is_empty()                        {return Ok(0)}
        let write_offset: u32 = u32::try_from(offset      ).map_err(|_| ReturnCode::VolumeOutOfBounds)?;
        let buffer_len:   u32 = u32::try_from(buffer.len()).map_err(|_| ReturnCode::BufferTooLarge)?;
        //if write_offset+buffer_len > self.file_size {return Err(ReturnCode::VolumeOutOfBounds)}
        //Calculate cluster and byte offsets into volume
        let start_cluster_offset: u32 = write_offset % self.cluster_size;
        let start_index:          u32 = write_offset / self.cluster_size;
        let end_index:            u32 = (write_offset + buffer_len - 1) / self.cluster_size;
        let end_cluster_offset:   u32 = (write_offset + buffer_len - 1) % self.cluster_size + 1;
        //Move to first cluster which needs to be read from
        let mut current_cluster: u32 = self.start_cluster;
        for _ in 0..start_index {
            //Retrieve next cluster
            current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                FATTableEntry::Used(cluster) => cluster as u32,
                FATTableEntry::End => {return Err(ReturnCode::EndOfVolume)}
                _ => {return Err(ReturnCode::SeekError)}
            };
        }
        //Start writing
        let mut buffer_offset: usize = 0;
        for index in start_index..=end_index {
            //If only one cluster needs to be accessed
            if index == start_index && index == end_index {
                //Write Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size + start_cluster_offset;
                return self.volume.write(volume_offset as u64, buffer).map_err(|_| ReturnCode::Test00);
            }
            //If its in the first cluster (may need to truncate)
            else if index == start_index {
                //Write Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size + start_cluster_offset;
                let new_buffer_offset: usize = (self.cluster_size - start_cluster_offset) as usize;
                return_if_partial!(0, self.volume.write_check(volume_offset as u64, &buffer[0..new_buffer_offset]));
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    FATTableEntry::End => {return Ok(buffer_offset as u64)},
                    _ => {return Err(ReturnCode::SeekError)}
                };
            }
            //If its in the final cluster (may need to truncate)
            else if index == end_index {
                //Write Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size;
                let new_buffer_offset: usize = buffer_offset+end_cluster_offset as usize;
                return_if_partial!(buffer_offset as u64, self.volume.write_check(volume_offset as u64, &buffer[buffer_offset..new_buffer_offset]));
                buffer_offset = new_buffer_offset;
            }
            //If its in the middle (always reads entire cluster)
            else {
                //Write data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size;
                let new_buffer_offset: usize = buffer_offset + self.cluster_size as usize;
                return_if_partial!(buffer_offset as u64, self.volume.write_check(volume_offset as u64, &buffer[buffer_offset..new_buffer_offset]));
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    FATTableEntry::End => {return Ok(buffer_offset as u64)},
                    _ => {return Err(ReturnCode::SeekError)}
                };
            }
        }
        Ok(buffer_offset as u64)
    }
}
/*impl <'s, RO: 's+Volume>                             FATFile<'s, RO> {
    fn file_offset_to_volume_offset (&self, offset: u32) -> Result<u32, ReturnCode> {
        //if offset > self.file_size {return Err(ReturnCode::VolumeOutOfBounds)}
        let index = offset / self.cluster_size;
        let final_offset = offset % self.cluster_size;
        let mut current_cluster: u32 = self.start_cluster;
        for _ in 0..index {
            current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                FATTableEntry::Used(cluster) => cluster as u32,
                _ => {return Err(ReturnCode::SeekError)}
            };
        }
        Ok(self.data_area_offset + (current_cluster - 2) * self.cluster_size + final_offset)
    }
}*/
/*impl <'s, RO: 's+VolumeRead>             FileRead    for FATFile<'s, RO> {
    // FILE READ ROUTINES
    fn get_name<'f>  (&self, buffer: &'f mut [u8]) -> Result<&'f str,  &'static str> {
        if buffer.len() < 12 {return Err("FAT16 File: Get name provided buffer of insufficient size.")}
        let mut directory_buffer: [u8; 0x20] = [0u8; 0x20];
        self.volume.read(self.directory_entry_pointer as usize, &mut directory_buffer)?;
        let directory_entry: FATDirectoryEntry = FATDirectoryEntry::try_from(directory_buffer)?;
        let name_raw = directory_entry.file_name;
        let name_array = &name_raw[0..8];
        let ext_array = &name_raw[8..11];
        let mut index = 0;
        for i in 0..8 {
            let ascii = name_array[i];
            if !(0x20..=0x7E).contains(&ascii) {return Err("FAT16 File: Get name found invalid characters.")}
            if ascii == 0x20 {break}
            buffer[index] = ascii;
            index += 1;
        }
        buffer[index] = 0x2E;
        index += 1;
        for i in 0..3 {
            let ascii = ext_array[i];
            if !(0x20..=0x7E).contains(&ascii) {return Err("FAT16 File: Get name found invalid characters.")}
            if ascii == 0x20 {break}
            buffer[index] = ascii;
            index += 1;
        }
        let ret_str = str::from_utf8(&buffer[0..index]).map_err(|_| "FAT16 File: Get name encountered error converting raw filename.")?;
        Ok(ret_str)
    }
    fn get_size      (&self)                       -> Result<usize,    &'static str> {
        Ok(self.file_size as usize)
    }
    fn get_timestamp (&self)                       -> Result<i64,      &'static str> {
        /*let mut directory_buffer: [u8; 0x20] = [0u8; 0x20];
        self.volume.read(self.directory_entry_pointer as usize, &mut directory_buffer)?;
        let directory_entry: FATDirectoryEntry = FATDirectoryEntry::try_from(directory_buffer)?;*/
        Err("FAT16 File: Read timestamp not yet implemented.")
    }
    fn get_write     (&self)                       -> Result<bool,     &'static str> {
        let mut directory_buffer: [u8; 0x20] = [0u8; 0x20];
        self.volume.read(self.directory_entry_pointer as usize, &mut directory_buffer)?;
        let directory_entry: FATDirectoryEntry = FATDirectoryEntry::try_from(directory_buffer)?;
        Ok(!directory_entry.file_attributes.read_only)
    }
    fn get_hidden    (&self)                       -> Result<bool,     &'static str> {
        let mut directory_buffer: [u8; 0x20] = [0u8; 0x20];
        self.volume.read(self.directory_entry_pointer as usize, &mut directory_buffer)?;
        let directory_entry: FATDirectoryEntry = FATDirectoryEntry::try_from(directory_buffer)?;
        Ok(!directory_entry.file_attributes.hidden)
    }
}
impl <'s, RW: 's+VolumeRead+VolumeWrite> FileWrite   for FATFile<'s, RW> {
    fn set_name      (&self, name_in: &str)        -> Result<(),       &'static str> {
        //Check name string is valid
        if !name_in.is_ascii() {return Err("FAT16 File: Set name given invalid characters.")}
        let name_bytes = name_in.as_bytes();
        if name_bytes.len() > 12 {return Err("FAT16 File: Set name provided a name string which is too long.")}
        //Find delimiter between name and extension
        let mut delimiter_index: Option<usize> = None;
        for i in 0..name_in.len() {
            if name_bytes[i] == 0x2E {delimiter_index = Some(i)}
        }
        //Determine name and extension from array
        let name_array = match delimiter_index {
            Some(i) => &name_bytes[0..i],
            None => name_bytes,
        };
        let ext_array = match delimiter_index {
            Some(i) => &name_bytes[i+1..],
            None => &[],
        };
        //Check name and extension are valid
        if name_array.len() > 8 {return Err("FAT16 File: Set name provided a name which is too long.")}
        if ext_array.len()  > 3 {return Err("FAT16 File: Set name provided an extension which is too long.")}
        //Create name array
        let mut name_final = [0x20u8; 11];
        name_final[0..name_array.len()].clone_from_slice(name_array);
        name_final[8..8+ext_array.len()].clone_from_slice(ext_array);
        //Load directory entry
        let mut directory_buffer: [u8; 0x20] = [0u8; 0x20];
        self.volume.read(self.directory_entry_pointer as usize, &mut directory_buffer)?;
        let mut directory_entry: FATDirectoryEntry = FATDirectoryEntry::try_from(directory_buffer)?;
        //Change directory entry
        directory_entry.file_name = name_final;
        let directory_array = <[u8;0x20]>::try_from(directory_entry)?;
        //Write directory entry
        self.volume.write(self.directory_entry_pointer as usize, &directory_array)?;
        Ok(())
    }
    fn set_size      (&self, _size: usize)         -> Result<(),       &'static str> {
        Err("FAT16 File: Set file size not yet implemented.")
    }
    fn set_timestamp (&self, _timestamp: i64)      -> Result<(),       &'static str> {
        Err("FAT16 File: Set timestamp not yet implemented.")
    }
    fn set_write     (&self, write: bool)          -> Result<(),       &'static str> {
        //Load directory entry
        let mut directory_buffer: [u8; 0x20] = [0u8; 0x20];
        self.volume.read(self.directory_entry_pointer as usize, &mut directory_buffer)?;
        let mut directory_entry: FATDirectoryEntry = FATDirectoryEntry::try_from(directory_buffer)?;
        //Change directory entry
        directory_entry.file_attributes.read_only = !write;
        let directory_array = <[u8;0x20]>::try_from(directory_entry)?;
        //Write directory entry
        self.volume.write(self.directory_entry_pointer as usize, &directory_array)?;
        Ok(())
    }
    fn set_hidden    (&self, hidden: bool)         -> Result<(),       &'static str> {
        //Load directory entry
        let mut directory_buffer: [u8; 0x20] = [0u8; 0x20];
        self.volume.read(self.directory_entry_pointer as usize, &mut directory_buffer)?;
        let mut directory_entry: FATDirectoryEntry = FATDirectoryEntry::try_from(directory_buffer)?;
        //Change directory entry
        directory_entry.file_attributes.hidden = hidden;
        let directory_array = <[u8;0x20]>::try_from(directory_entry)?;
        //Write directory entry
        self.volume.write(self.directory_entry_pointer as usize, &directory_array)?;
        Ok(())
    }
}*/
