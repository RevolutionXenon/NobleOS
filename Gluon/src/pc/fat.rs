// GLUON: PC FILE ALLOCATION TABLE
// Structs and enums related to the contents and handling of the FAT16 file system


// HEADER
//Flags
#![allow(clippy::needless_range_loop)]

//Imports
use crate::numeric_enum;
use crate::noble::file_system::*;
use core::{convert::{TryFrom, TryInto}};
use core::str;


// FAT16 FILE SYSTEMS
//Full FAT16 Handling Routines
pub struct FATFileSystem<'s, V: 's> {
    pub volume:     &'s V,
    pub boot_sector:    FATBootSector,
    pub fat:            FATTable<'s, V>,
    pub root_directory: FATDirectory<VolumeFromVolume<'s, V>>,
}
impl<'s, WO: 's+VolumeWrite> FATFileSystem<'s, WO> {
    // CONSTRUCTOR
    pub fn format_new(volume: &'s WO, boot_sector: FATBootSector) -> Result <Self, &'static str> {
        //Clear
        for i in 0..(boot_sector.first_data_sector() * boot_sector.bytes_per_sector as u32) as usize / 0x200 {  
            volume.write(0x200*i, &[0u8; 0x200])?;
        }
        //Write Boot Sector
        volume.write(0, &<[u8;512]>::try_from(boot_sector)?)?;
        //Create FAT
        let fat = FATTable::new(volume, boot_sector);
        fat.write_raw(0, 0xFFF0)?;
        fat.write_raw(1, 0xFFFF)?;
        //Create Root Directory
        let root_directory = FATDirectory {
            directory: VolumeFromVolume {
                volume,
                offset: (boot_sector.first_root_sector() * boot_sector.bytes_per_sector as u32) as usize,
                size: boot_sector.root_entry_count as usize * 32,
            },
            num_entries: boot_sector.root_entry_count as u32,
        };
        //Return
        Ok(Self {volume, boot_sector, fat, root_directory})
    }
}
impl<'s, RO: 's+VolumeRead>  FATFileSystem<'s, RO> {
    // CONSTRUCTOR
    pub fn from_existing_volume(volume: &'s RO) -> Result<Self, &'static str> {
        //Load Boot Sector
        let mut buffer: [u8; 0x200] = [0u8; 0x200];
        volume.read(0x00, &mut buffer)?;
        let boot_sector = FATBootSector::try_from(buffer)?;
        //Load FAT
        let fat = FATTable::new(volume, boot_sector);
        //Load Root Directory
        let root_directory = FATDirectory {
            directory: VolumeFromVolume {
                volume,
                offset: (boot_sector.first_root_sector() * boot_sector.bytes_per_sector as u32) as usize,
                size: boot_sector.root_entry_count as usize * 32,
            },
            num_entries: boot_sector.root_entry_count as u32,
        };
        //Return
        Ok(Self {volume, boot_sector, fat, root_directory})
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
    pub fn fat_location      (&self) -> u32 {self.first_fat_sector() * self.bytes_per_sector as u32}
    pub fn fat_size          (&self) -> u32 {self.sectors_per_fat as u32 * self.bytes_per_sector as u32}
    pub fn first_root_sector (&self) -> u32 {self.first_fat_sector() + (self.sectors_per_fat as u32 * self.fat_number as u32)}
    pub fn root_location     (&self) -> u32 {self.first_root_sector() * self.bytes_per_sector as u32}
    pub fn first_data_sector (&self) -> u32 {self.first_root_sector() + ((self.root_entry_count as u32 * 32) / self.bytes_per_sector as u32)}
    pub fn data_location     (&self) -> u32 {self.first_data_sector() * self.bytes_per_sector as u32}
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
pub struct FATTable<'s, RW: 's> {
    volume:     &'s RW,
    start_location: u32,
    entry_count:    u32,
    fat_count:      u32,
    fat_size:       u32,
}
impl<'s, RW: 's>             FATTable<'s, RW> {
    // CONSTRUCTOR
    pub fn new(volume: &'s RW, boot_sector: FATBootSector) -> Self {
        Self {
            volume,
            start_location: boot_sector.fat_location(),
            entry_count:    (boot_sector.total_sectors() - boot_sector.first_data_sector()) / boot_sector.sectors_per_cluster as u32 + 2,
            fat_count:      boot_sector.fat_number as u32,
            fat_size:       boot_sector.fat_size()
        }
    }
}
impl<'s, RW: 's+VolumeRead>  FATTable<'s, RW> {
    // READ ONLY
    pub fn read_entry(&self, cluster: u16) -> Result<FATTableEntry, &'static str> {
        if cluster as u32 > self.entry_count {return Err("FAT 16 Table Read Entry: Cluster index out of bounds.")}
        let mut buffer = [0u8;2];
        let table_offset = (cluster * 2) as u32;
        self.volume.read((self.start_location + table_offset) as usize, &mut buffer)?;
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
}
impl<'s, RW: 's+VolumeWrite> FATTable<'s, RW> {
    // WRITE ONLY
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
            self.volume.write((self.start_location + (i*self.fat_size) + table_offset) as usize, &entry_bytes)?;
        }
        Ok(())
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
pub struct FATDirectory<V> {
    pub directory:     V,
    pub num_entries:   u32,
}
impl<RW: VolumeRead+VolumeWrite> FATDirectory<RW> {
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
        let mut bytes = [0u8;0x20];
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
    pub reserved_6:      bool, //Bit  6
    pub reserved_7:      bool, //Bit  7
    pub read_password:   bool, //Bit  8
    pub write_password:  bool, //Bit  9
    pub delete_password: bool, //Bit 10
    pub reserved_b:      bool, //Bit 11
    pub reserved_c:      bool, //Bit 12
    pub reserved_d:      bool, //Bit 13
    pub reserved_e:      bool, //Bit 14
    pub reserved_f:      bool, //Bit 15
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
            read_password:   false,
            write_password:  false,
            delete_password: false,
            reserved_b:      false,
            reserved_c:      false,
            reserved_d:      false,
            reserved_e:      false,
            reserved_f:      false,
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
            read_password:   false,
            write_password:  false,
            delete_password: false,
            reserved_b:      false,
            reserved_c:      false,
            reserved_d:      false,
            reserved_e:      false,
            reserved_f:      false,
        }
    }
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
            reserved_6:      bytes[0] & 0b0100_0000 > 0,
            reserved_7:      bytes[0] & 0b1000_0000 > 0,
            read_password:   bytes[1] & 0b0000_0001 > 0,
            write_password:  bytes[1] & 0b0000_0010 > 0,
            delete_password: bytes[1] & 0b0000_0100 > 0,
            reserved_b:      bytes[0] & 0b0000_1000 > 0,
            reserved_c:      bytes[0] & 0b0001_0000 > 0,
            reserved_d:      bytes[0] & 0b0010_0000 > 0,
            reserved_e:      bytes[0] & 0b0100_0000 > 0,
            reserved_f:      bytes[0] & 0b1000_0000 > 0,
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
         if attributes.archive         {0b0010_0000} else {0} +
         if attributes.reserved_6      {0b0100_0000} else {0} +
         if attributes.reserved_7      {0b1000_0000} else {0},
         if attributes.read_password   {0b0000_0001} else {0} +
         if attributes.write_password  {0b0000_0010} else {0} +
         if attributes.delete_password {0b0000_0100} else {0} +
         if attributes.reserved_b      {0b0000_1000} else {0} +
         if attributes.reserved_c      {0b0001_0000} else {0} +
         if attributes.reserved_d      {0b0010_0000} else {0} +
         if attributes.reserved_e      {0b0100_0000} else {0} +
         if attributes.reserved_f      {0b1000_0000} else {0}]
    }
}


// FILE HANDLE
//FAT File
pub struct FATFile<'s, V: 's> {
    volume:              &'s V,
    fat:                 &'s FATTable<'s, V>,
    directory_entry_pointer: u32,
    data_area_offset:        u32,
    start_cluster:           u32,
    cluster_size:            u32,
    file_size:               u32,
}
impl <'s, RW: 's>                                        FATFile<'s, RW> {
    // CONSTRUCTOR
    pub fn new_from_start_cluster(fs: &'s FATFileSystem<RW>, directory_entry_pointer: u32, start_cluster: u32, file_size: u32) -> Result<Self, &'static str> {
        let bs = fs.boot_sector;
        let fat = &fs.fat;
        Ok(Self {
            volume: fs.volume,
            fat,
            directory_entry_pointer,
            data_area_offset: bs.first_data_sector() * bs.bytes_per_sector as u32,
            start_cluster,
            cluster_size: bs.cluster_size(),
            file_size,
        })
    }
}
impl <'s, RO: 's+VolumeRead>             VolumeRead  for FATFile<'s, RO> {
    // READ ONLY
    fn read (&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
        //Test bounds to ensure read won't fail
        if buffer.is_empty()                       {return Ok(())}
        let read_offset: u32 = u32::try_from(offset      ).map_err(|_| "FAT16 File: Offset larger than 4GiB requested.")?;
        let buffer_len:  u32 = u32::try_from(buffer.len()).map_err(|_| "FAT16 File: Buffer larger than 4GiB requested.")?;
        if read_offset+buffer_len > self.file_size {return Err("FAT16 File: Out of bounds on read.")}
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
                _ => {return Err("FAT16 File: Cluster error during read seek (corrupted file).")}
            };
        }
        //Start reading
        let mut buffer_offset: usize = 0;
        for index in start_index..=end_index {
            //If only one cluster needs to be accessed
            if index == start_index && index == end_index {
                //Read Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size + start_cluster_offset;
                self.volume.read(volume_offset as usize, buffer)?;
            }
            //If its in the first cluster (may need to truncate)
            else if index == start_index {
                //Read Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size + start_cluster_offset;
                let new_buffer_offset: usize = (self.cluster_size - start_cluster_offset) as usize;
                self.volume.read(volume_offset as usize, &mut buffer[0..new_buffer_offset])?;
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    _ => {return Err("FAT16 File: Cluster error during first sector read (corrupted file).")}
                };
            }
            //If its in the final cluster (may need to truncate)
            else if index == end_index {
                //Read Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size;
                self.volume.read(volume_offset as usize, &mut buffer[buffer_offset..buffer_offset+end_cluster_offset as usize])?;
            }
            //If its in the middle (always reads entire cluster)
            else {
                //Read Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size;
                let new_buffer_offset: usize = buffer_offset + self.cluster_size as usize;
                self.volume.read(volume_offset as usize, &mut buffer[buffer_offset..new_buffer_offset])?;
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    _ => {return Err("FAT16 File: Cluster error during mid sector read (corrupted file).")}
                };
            }
        }
        Ok(())
    }
}
impl <'s, RW: 's+VolumeRead+VolumeWrite> VolumeWrite for FATFile<'s, RW> {
    // WRITE
    fn write(&self, offset: usize, buffer: &[u8])     -> Result<(), &'static str> {
        //Test bounds to ensure read won't fail
        if buffer.is_empty()                        {return Ok(())}
        let write_offset: u32 = u32::try_from(offset      ).map_err(|_| "FAT16 File: Offset larger than 4GiB requested.")?;
        let buffer_len:  u32 = u32::try_from(buffer.len()).map_err(|_| "FAT16 File: Buffer larger than 4GiB requested.")?;
        if write_offset+buffer_len > self.file_size {return Err("FAT16 File: Out of bounds on write.")}
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
                _ => {return Err("FAT16 File: Cluster error during write seek (corrupted file).")}
            };
        }
        //Start writing
        let mut buffer_offset: usize = 0;
        for index in start_index..=end_index {
            //If only one cluster needs to be accessed
            if index == start_index && index == end_index {
                //Write Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size + start_cluster_offset;
                self.volume.write(volume_offset as usize, buffer)?;
            }
            //If its in the first cluster (may need to truncate)
            else if index == start_index {
                //Write Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size + start_cluster_offset;
                let new_buffer_offset: usize = (self.cluster_size - start_cluster_offset) as usize;
                self.volume.write(volume_offset as usize, &buffer[0..new_buffer_offset])?;
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    _ => {return Err("FAT16 File: Cluster error during first sector write (corrupted file).")}
                };
            }
            //If its in the final cluster (may need to truncate)
            else if index == end_index {
                //Write Data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size;
                self.volume.write(volume_offset as usize, &buffer[buffer_offset..buffer_offset+end_cluster_offset as usize])?;
            }
            //If its in the middle (always reads entire cluster)
            else {
                //Write data
                let volume_offset: u32 = self.data_area_offset + (current_cluster - 2) * self.cluster_size;
                let new_buffer_offset: usize = buffer_offset + self.cluster_size as usize;
                self.volume.write(volume_offset as usize, &buffer[buffer_offset..new_buffer_offset])?;
                buffer_offset = new_buffer_offset;
                //Retrieve next cluster
                current_cluster = match self.fat.read_entry(current_cluster as u16)? {
                    FATTableEntry::Used(cluster) => cluster as u32,
                    _ => {return Err("FAT16 File: Cluster error during mid sector write (corrupted file).")}
                };
            }
        }
        Ok(())
    }
}
impl <'s, RO: 's+VolumeRead>             FileRead    for FATFile<'s, RO> {
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
}
