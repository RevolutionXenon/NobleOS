// GLUON: PC FILE ALLOCATION TABLE
// Structs and enums related to the contents and handling of the FAT16 file system


// HEADER
//Imports
use crate::*;
use core::{convert::{TryFrom, TryInto}};


// FAT16 FILE SYSTEMS
//Full FAT16 Handling Routines
pub struct FATFileSystem<'s, RW: 's+LocationalRead+LocationalWrite> {
    pub volume:  &'s RW,
    pub boot_sector: FATBootSector,
    pub fat:         FATTable<'s, RW>,
}
impl<'s, RW: 's+LocationalRead+LocationalWrite> FATFileSystem<'s, RW> {
    // CONSTRUCTOR
    pub fn new(volume: &'s RW, boot_sector: FATBootSector) -> Result <Self, &'static str> {
        //Create FAT
        let fat = FATTable::new(volume, boot_sector);
        //Return
        Ok(Self {volume, boot_sector, fat})
    }
    pub fn from_existing_volume(volume: &'s RW) -> Result<Self, &'static str> {
        //Load Boot Sector
        let mut buffer: [u8; 0x200] = [0u8; 0x200];
        volume.read(0x00, &mut buffer)?;
        let boot_sector = FATBootSector::try_from(buffer)?;
        //Load FAT
        let fat = FATTable::new(volume, boot_sector);
        //Return
        Ok(Self {volume, boot_sector, fat})
    }
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
    pub fn first_fat_sector  (&self) -> u32 {self.reserved_sector_count as u32}
    pub fn first_root_sector (&self) -> u32 {self.first_fat_sector() + (self.sectors_per_fat as u32 * self.fat_number as u32)}
    pub fn first_data_sector (&self) -> u32 {self.first_root_sector() + ((self.root_entry_count as u32 * 32) / self.bytes_per_sector as u32)}
}
impl TryFrom<[u8;0x200]> for FATBootSector {
    type Error = &'static str;

    fn try_from(bytes: [u8;512]) -> Result<Self, Self::Error> {
        if bytes[0x0026]        != 0x29         {return Err("FAT16 Boot Sector: Invalid extended boot signature.")}
        if bytes[0x01FE..0x200] != [0x55, 0xAA] {return Err("FAT16 Boot Sector: Invalid boot signature.")}
        Ok( Self {
            jump_instruction:                                                       bytes[0x0000..0x0003].try_into().map_err(|_| "FAT16 Boot Sector: Error converting jump instruction value.")?,
            oem_name:                                                               bytes[0x0003..0x000B].try_into().map_err(|_| "FAT16 Boot Sector: Error converting OEM name value.")?,
            bytes_per_sector:           BytesPerSector::try_from(u16::from_le_bytes(bytes[0x000B..0x000D].try_into().map_err(|_| "FAT16 Boot Sector: Error converting bytes per sector value.")?))
                                                                                                                    .map_err(|_| "FAT16 Boot Sector: Invalid bytes per sector value.")?,
            sectors_per_cluster:     SectorsPerCluster::try_from(                   bytes[0x000D])                  .map_err(|_| "FAT16 Boot Sector: Invalid sectors per cluster.")?,
            reserved_sector_count:                               u16::from_le_bytes(bytes[0x000E..0x0010].try_into().map_err(|_| "FAT16 Boot Sector: Error converting reserved sector count value.")?),
            fat_number:                                                             bytes[0x0010],
            root_entry_count:                                    u16::from_le_bytes(bytes[0x0011..0x0013].try_into().map_err(|_| "FAT16 Boot Sector: Error converting root entry count value.")?),
            total_sectors_16:                                    u16::from_le_bytes(bytes[0x0013..0x0015].try_into().map_err(|_| "FAT16 Boot Sector: Error converting total sectors (16) value.")?),
            media:                           MediaType::try_from(                   bytes[0x0015]                  ).map_err(|_| "FAT16 Boot Sector: Invalid media type.")?,
            sectors_per_fat:                                     u16::from_le_bytes(bytes[0x0016..0x0018].try_into().map_err(|_| "FAT16 Boot Sector: Error converting sectors per fat value.")?),
            sectors_per_track:                                   u16::from_le_bytes(bytes[0x0018..0x001A].try_into().map_err(|_| "FAT16 Boot Sector: Error converting sectors per track value.")?),
            heads_number:                                        u16::from_le_bytes(bytes[0x001A..0x001C].try_into().map_err(|_| "FAT16 Boot Sector: Error converting head count value.")?),
            hidden_sectors:                                      u32::from_le_bytes(bytes[0x001C..0x0020].try_into().map_err(|_| "FAT16 Boot Sector: Error converting hidden sector count value.")?),
            total_sectors_32:                                    u32::from_le_bytes(bytes[0x0020..0x0024].try_into().map_err(|_| "FAT16 Boot Sector: Error converting total sectors (32) value.")?),
            drive_number:                                                           bytes[0x0024],
            volume_id:                                           u32::from_le_bytes(bytes[0x0027..0x002B].try_into().map_err(|_| "FAT16 Boot Sector: Error converting drive number value.")?),
            volume_label:                                                           bytes[0x002B..0x0036].try_into().map_err(|_| "FAT16 Boot Sector: Error converting volume label value.")?,
            file_system_type:                                                       bytes[0x0036..0x003E].try_into().map_err(|_| "FAT16 Boot Sector: Error converting file system type value.")?,
            bootstrap_code:                                                         bytes[0x003E..0x01FE].try_into().map_err(|_| "FAT16 Boot Sector: Error converting bootstrap code.")?,
        })
    }
}
impl TryFrom<FATBootSector> for [u8;0x200] {
    type Error = &'static str;

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
pub struct FATTable<'s, RW: 's+LocationalRead+LocationalWrite> {
    volume:     &'s RW,
    start_location: u32,
    entry_count:    u32,
    fat_count:      u32,
    fat_size:       u32,
}
impl<'s, RW: 's+LocationalRead+LocationalWrite> FATTable<'s, RW> {
    // CONSTRUCTOR
    pub fn new(volume: &'s RW, boot_sector: FATBootSector) -> Self {
        let start_location = boot_sector.first_fat_sector() * boot_sector.bytes_per_sector as u32;
        let entry_count = (boot_sector.total_sectors() - boot_sector.first_data_sector()) / boot_sector.sectors_per_cluster as u32 + 2;
        let fat_count = boot_sector.fat_number as u32;
        let fat_size = boot_sector.sectors_per_fat as u32 * boot_sector.bytes_per_sector as u32;
        Self {volume, start_location, entry_count, fat_count, fat_size,}
    }

    // READ AND WRITE
    pub fn read_entry(&self, cluster: u16) -> Result<FATTableEntry, &'static str> {
        if cluster as u32 > self.entry_count {return Err("FAT 16 Table Read Entry: Cluster index out of bounds.")}
        let mut buffer = [0u8;2];
        let offset = self.start_location as usize + (2 * cluster as usize);
        self.volume.read(offset, &mut buffer)?;
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

    pub fn write_entry(&self, cluster: u16, entry: FATTableEntry) -> Result<(), &'static str> {
        if cluster as u32 > self.entry_count {return Err("FAT 16 Table Write Entry: Cluster index out of bounds.")}
        let table_offset = (cluster * 2) as u32;
        let entry_bytes = match entry {
            FATTableEntry::Free        => 0x0000,
            FATTableEntry::Reserved    => 0x0001,
            FATTableEntry::Used(v) => v,
            FATTableEntry::Bad         => 0xFFF7,
            FATTableEntry::End         => 0xFFFF,
        }.to_le_bytes();
        for i in 0..self.fat_count {
            self.volume.write((self.start_location + i*self.fat_size + table_offset) as usize, &entry_bytes)?;
        }
        Ok(())
    }
    pub fn write_raw  (&self, cluster: u16, entry: u16) -> Result<(), &'static str> {
        if cluster as u32 > self.entry_count {return Err("FAT 16 Table Write Entry: Cluster index out of bounds.")}
        let table_offset = (cluster * 2) as u32;
        let entry_bytes = entry.to_le_bytes();
        for i in 0..self.fat_count {
            self.volume.write((self.start_location + i*self.fat_size + table_offset) as usize, &entry_bytes)?;
        }
        Ok(())
    }
}

//File Allocation Table Entry
pub enum FATTableEntry {
    Free,
    Reserved,
    Used(u16),
    Bad,
    End,
}


// FAT DIRECTORY
//Directory
pub struct FATDirectory<'s, RW: 's+LocationalRead+LocationalWrite> {
    pub directory: &'s RW,
    pub num_entries:   u32,
}
impl<'s, RW: 's+LocationalRead+LocationalWrite> FATDirectory<'s, RW> {
    pub fn read_entry(&self, position: u32) -> Result<FATDirectoryEntry, &'static str> {
        if position >= self.num_entries {return Err("FAT16 Directory: Index out of bounds on read.")}
        FATDirectoryEntry::try_from({
            let mut buffer = [0u8;0x20];
            self.directory.read(position as usize * 32, &mut buffer)?;
            buffer
        })
    }

    pub fn write_entry(&self, position: u32, entry: FATDirectoryEntry) -> Result<(), &'static str> {
        let buffer: [u8; 0x20] = <[u8;0x20]>::try_from(entry)?;
        self.directory.write(position as usize * 32, &buffer)?;
        Ok(())
    }
}

//Directory Entry
pub struct FATDirectoryEntry {
    pub file_name:          [u8; 11],
    pub file_attributes:    FATFileAttributes,
    pub start_cluster_high: u16,
    pub start_cluster_low:  u16,
    pub creation_time_ss:   u8,
    pub creation_time:      u16,
    pub creation_date:      u16,
    pub file_size:          u32,
}
impl FATDirectoryEntry {
    pub fn start_cluster_16(&self) -> Result<u16, &'static str> {
        if self.start_cluster_high > 0 {return Err("FAT 16 Directory Entry: 16-bit start cluster requested when directory holds 32-bit entry.")}
        Ok(self.start_cluster_low)
    }
    pub fn start_cluster_32(&self) -> u32 {
        ((self.start_cluster_high as u32) << 16) + self.start_cluster_low as u32
    }
}
impl TryFrom<[u8;0x20]> for FATDirectoryEntry {
    type Error = &'static str;

    fn try_from(bytes: [u8;0x20]) -> Result<Self, Self::Error> {
        Ok(Self {
            file_name:                                                                 bytes[0x00..0x0B].try_into().map_err(|_| "FAT16 Directory Entry: Error converting file name value.")?,
            file_attributes: FATFileAttributes::from(TryInto::<[u8;2]>::try_into(&bytes[0x0B..0x0D]          ).map_err(|_| "FAT16 Directory Entry: Error converting file attributes value.")?),
            creation_time_ss:                                                          bytes[0x0D],
            start_cluster_high:                                     u16::from_le_bytes(bytes[0x14..0x16].try_into().map_err(|_| "FAT16 Directory Entry: Error converting cluster high value.")?),
            creation_time:                                          u16::from_le_bytes(bytes[0x16..0x18].try_into().map_err(|_| "FAT16 Directory Entry: Error converting creation subsecond value.")?),
            creation_date:                                          u16::from_le_bytes(bytes[0x18..0x1A].try_into().map_err(|_| "FAT16 Directory Entry: Error converting creation time value.")?),
            start_cluster_low:                                      u16::from_le_bytes(bytes[0x1A..0x1C].try_into().map_err(|_| "FAT16 Directory Entry: Error converting creation date value.")?),
            file_size:                                              u32::from_le_bytes(bytes[0x1C..0x20].try_into().map_err(|_| "FAT16 Directory Entry: Error converting cluster low value.")?),
        })
    }
}
impl TryFrom<FATDirectoryEntry> for [u8;0x20] {
    type Error = &'static str;

    fn try_from(entry: FATDirectoryEntry) -> Result<Self, Self::Error> {
        if entry.creation_time_ss >= 200 {return Err("FAT16 Directory Entry: Invalid timestamp.")}
        let mut bytes = [0u8;32];
        bytes[0x00..0x0B].clone_from_slice(&entry.file_name);
        bytes[0x0B..0x0D].clone_from_slice(&<[u8;2]>::from(entry.file_attributes));
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
pub struct FATFileAttributes {
    pub read_only:       bool, //Bit  0
    pub hidden:          bool, //Bit  1
    pub system:          bool, //Bit  2
    pub volume_label:    bool, //Bit  3
    pub subdirectory:    bool, //Bit  4
    pub archive:         bool, //Bit  5
    pub read_password:   bool, //Bit  8
    pub write_password:  bool, //Bit  9
    pub delete_password: bool, //Bit 10
}
impl From<[u8;2]> for FATFileAttributes {
    fn from(bytes: [u8;2]) -> Self {
        Self {
            read_only:       bytes[0] & 0b0000_0001 > 0,
            hidden:          bytes[0] & 0b0000_0010 > 0,
            system:          bytes[0] & 0b0000_0100 > 0,
            volume_label:    bytes[0] & 0b0000_1000 > 0,
            subdirectory:    bytes[0] & 0b0001_0000 > 0,
            archive:         bytes[0] & 0b0010_0000 > 0,
            read_password:   bytes[1] & 0b0000_0001 > 0,
            write_password:  bytes[1] & 0b0000_0010 > 0,
            delete_password: bytes[1] & 0b0000_0100 > 0,
        }
    }
}
impl From<FATFileAttributes> for [u8;2] {
    fn from(attributes: FATFileAttributes) -> Self {
        [if attributes.read_only       {0b0000_0001} else {0} +
         if attributes.hidden          {0b0000_0010} else {0} +
         if attributes.system          {0b0000_0100} else {0} +
         if attributes.volume_label    {0b0000_1000} else {0} +
         if attributes.subdirectory    {0b0001_0000} else {0} +
         if attributes.archive         {0b0010_0000} else {0},
         if attributes.read_password   {0b0000_0001} else {0} +
         if attributes.write_password  {0b0000_0010} else {0} +
         if attributes.delete_password {0b0000_0100} else {0}]
    }
}


// FILE HANDLE
pub struct FATFile<'s, RW: 's+LocationalRead+LocationalWrite> {
    volume:       &'s RW,
    fat:          &'s FATTable<'s, RW>,
    data_area_offset: u32,
    start_cluster:    u32,
    cluster_size:     u32,
    file_size:        u32,
}
impl <'s, RW: 's+LocationalRead+LocationalWrite> FATFile<'s, RW> {
    pub fn new_from_start_cluster(fs: &'s FATFileSystem<RW>, start_cluster: u32, file_size: u32) -> Result<Self, &'static str> {
        let bs = fs.boot_sector;
        let fat = &fs.fat;
        Ok(Self {
            volume: fs.volume,
            fat,
            data_area_offset: bs.first_data_sector() * bs.bytes_per_sector as u32,
            start_cluster,
            cluster_size: bs.cluster_size(),
            file_size,
        })
    }
}
impl <'s, RW: 's+LocationalRead+LocationalWrite> LocationalRead for FATFile<'s, RW> {
    fn read (&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
        //Tests
        if offset              >= 2^32           {return Err("FAT16 File: Offset larger than 4GiB requested.")}
        if buffer.len()        >= 2^32           {return Err("FAT16 File: Buffer larger than 4GiB requested.")}
        let read_offset = offset as u32;
        let buffer_len = buffer.len() as u32;
        if read_offset + buffer_len >= self.file_size {return Err("FAT16 File: Out of bounds on read.")}
        //Offsets
        let start_cluster_offset = read_offset % self.cluster_size;
        let start_index = read_offset / self.cluster_size;
        let end_index = read_offset + buffer_len / self.cluster_size;
        let end_cluster_offset = {let temp = read_offset + buffer_len % self.cluster_size; if temp == 0 {self.cluster_size} else {temp}};
        //Start reading
        let mut current_cluster: u32 = self.start_cluster;
        let mut buffer_offset: usize = 0;
        for index in start_index..end_index+1 {
            if index == start_index && index == end_index {
                //Read Data
                let offset: u32 = self.data_area_offset + current_cluster * self.cluster_size + start_cluster_offset;
                self.volume.read(offset as usize, buffer)?;
            }
            else if index == start_index {
                //Read Data
                let offset: u32 = self.data_area_offset + current_cluster * self.cluster_size + start_cluster_offset;
                let new_buffer_offset: usize = (self.cluster_size - start_cluster_offset) as usize;
                self.volume.read(offset as usize, &mut buffer[0..new_buffer_offset])?;
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    _ => {return Err("FAT16 File: Cluster error.")}
                };
            }
            else if index == end_index {
                //Read Data
                let offset: u32 = self.data_area_offset + current_cluster * self.cluster_size;
                self.volume.read(offset as usize, &mut buffer[buffer_offset..buffer_offset+end_cluster_offset as usize])?;
            }
            else {
                //Read Data
                let offset: u32 = self.data_area_offset + current_cluster * self.cluster_size;
                let new_buffer_offset: usize = buffer_offset + self.cluster_size as usize;
                self.volume.read(offset as usize, &mut buffer[buffer_offset..new_buffer_offset])?;
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    _ => {return Err("FAT16 File: Cluster error.")}
                };
            }
        }
        Ok(())
    }
}
