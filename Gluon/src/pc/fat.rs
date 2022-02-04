// GLUON: PC FILE ALLOCATION TABLE
// Structs and enums related to the contents and handling of the FAT16 file system


// HEADER
//Imports
use crate::*;
use core::convert::{TryFrom, TryInto};


// FAT16 FILE SYSTEMS
//Full FAT16 Handling Routines
pub struct FileAllocationTable16<'a, LR: 'a+LocationalRead> {
    pub volume:  &'a LR,
    pub boot_sector: BootSector16,
}
impl<'a, LR: 'a+LocationalRead> FileAllocationTable16<'a, LR> {
}


// FAT12 / FAT16 BOOT SECTOR
//Boot Sector
#[derive(Debug)]
pub struct BootSector16 {
    pub jump_instruction:     [ u8; 0x0003], //Jump instruction
    pub oem_name:             [ u8; 0x0008], //Name string, usually "MSWIN4.1"
    pub bytes_per_sector:      BytesPerSector,      //512, 1024, 2048, or 4096 only
    pub sectors_per_cluster:   SectorsPerCluster,   //Power of 2 greater than 2^0, should not result in a bytes per cluster > 32KB
    pub reserved_sector_count: ReservedSectorCount,
    pub fat_number:             u8,     //Should be 2
    pub root_entry_count:      u16,     //FAT16: number of 32-byte entries in
    pub total_sectors_16:      u16,
    pub media:                 MediaType,
    pub fat_size_16:           u16,
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
impl TryFrom<&[u8]> for BootSector16 {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len()          <  0x0200       {return Err("FAT16 Boot Sector: Length of data given to parse from not large enough to contain boot sector.")}
        if bytes[0x0026]        != 0x29         {return Err("FAT16 Boot Sector: Invalid extended boot signature.")}
        if bytes[0x01FE..0x200] != [0x55, 0xAA] {return Err("FAT16 Boot Sector: Invalid boot signature.")}
        Ok( Self {
            jump_instruction:                                                       bytes[0x0000..0x0003].try_into().unwrap(),
            oem_name:                                                               bytes[0x0003..0x000B].try_into().unwrap(),
            bytes_per_sector:           BytesPerSector::try_from(u16::from_le_bytes(bytes[0x000B..0x000D].try_into().unwrap())).map_err(|_| "FAT16 Boot Sector: Invalid bytes per sector value.")?,
            sectors_per_cluster:     SectorsPerCluster::try_from(                   bytes[0x000D]                             ).map_err(|_| "FAT16 Boot Sector: Invalid sectors per cluster.")?,
            reserved_sector_count: ReservedSectorCount::try_from(u16::from_le_bytes(bytes[0x000E..0x0010].try_into().unwrap())).map_err(|_| "FAT16 Boot Sector: Invalid reserved sector count.")?,
            fat_number:                                                             bytes[0x0010],
            root_entry_count:                                    u16::from_le_bytes(bytes[0x0011..0x0013].try_into().unwrap()),
            total_sectors_16:                                    u16::from_le_bytes(bytes[0x0013..0x0015].try_into().unwrap()),
            media:                           MediaType::try_from(                   bytes[0x0015]                             ).map_err(|_| "FAT16 Boot Sector: Invalid media type.")?,
            fat_size_16:                                         u16::from_le_bytes(bytes[0x0016..0x0018].try_into().unwrap()),
            sectors_per_track:                                   u16::from_le_bytes(bytes[0x0018..0x001A].try_into().unwrap()),
            heads_number:                                        u16::from_le_bytes(bytes[0x001A..0x001C].try_into().unwrap()),
            hidden_sectors:                                      u32::from_le_bytes(bytes[0x001C..0x0020].try_into().unwrap()),
            total_sectors_32:                                    u32::from_le_bytes(bytes[0x0020..0x0024].try_into().unwrap()),
            drive_number:                                                           bytes[0x0024],
            volume_id:                                           u32::from_le_bytes(bytes[0x0027..0x002B].try_into().unwrap()),
            volume_label:                                                           bytes[0x002B..0x0036].try_into().unwrap(),
            file_system_type:                                                       bytes[0x0036..0x003E].try_into().unwrap(),
            bootstrap_code:                                                         bytes[0x003E..0x01FE].try_into().unwrap(),
        })
    }
}
impl TryFrom<BootSector16> for [u8;0x200] {
    type Error = &'static str;

    fn try_from(sector: BootSector16) -> Result<Self, Self::Error> {
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
        bytes[0x0016..0x0018].clone_from_slice(&sector.fat_size_16.to_le_bytes());
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

//Reserved Sector Count
numeric_enum! {
    #[repr(u16)]
    #[derive(PartialEq)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum ReservedSectorCount {
        FAT16 = 0x0001,
        FAT32 = 0x0020,
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
