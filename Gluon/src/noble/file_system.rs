// GLUON: NOBLE FILE SYSTEM
// Structs and traits for handling file systems in a generic manner

// HEADER
//Imports
use core::arch::asm;


// TRAITS
//Volume Read
pub trait VolumeRead  {
    fn read  (&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str>;
}
impl<'a, T: 'a + VolumeRead>  VolumeRead  for &'a T {
    fn read  (&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
        (**self).read(offset, buffer)
    }
}
//Volume Write
pub trait VolumeWrite {
    fn write (&self, offset: usize, buffer: &[u8])     -> Result<(), &'static str>;
}
impl<'a, T: 'a + VolumeWrite> VolumeWrite for &'a T {
    fn write  (&self, offset: usize, buffer: &[u8])    -> Result<(), &'static str> {
        (**self).write(offset, buffer)
    }
}

//File System Read Handle
pub trait FileSystemRead  {
    //Associated Types
    type File: FileRead;
    type Directory: DirectoryRead;
    //Find File/Directory
    fn retrieve_file      (&self, path: &str, delimiter: char) -> Result<Self::File, &'static str>;
    fn retrieve_directory (&self, path: &str, delimiter: char) -> Result<Self::Directory, &'static str>;
}
//File System Write Handle
pub trait FileSystemWrite {
    //Associated Types
    type File: FileWrite;
    type Directory: DirectoryWrite;
    //New File/Directory
    fn create_file      (&self, size: usize, path: &str, delimiter: char) -> Result<Self::File, &'static str>;
    fn create_directory (&self, path: &str, delimiter: char) -> Result<Self::Directory, &'static str>;
    //Delete File/Directory
    fn delete_file      (&self, path: &str, delimiter: char) -> Result<(), &'static str>;
    fn delete_directory (&self, path: &str, delimiter: char) -> Result<(), &'static str>;
}

//Directory Read Handle
pub trait DirectoryRead  {
    //Get List of Files (iterator?)
}
//Directory Write Handle
pub trait DirectoryWrite {
    //???
}

//File Read Handle
pub trait FileRead    {
    fn get_name<'f>  (&self, buffer: &'f mut [u8]) -> Result<&'f str,  &'static str>;
    fn get_size      (&self)                       -> Result<usize,    &'static str>;
    fn get_timestamp (&self)                       -> Result<i64,      &'static str>;
    fn get_write     (&self)                       -> Result<bool,     &'static str>;
    fn get_hidden    (&self)                       -> Result<bool,     &'static str>;
}
//File Write Handle
pub trait FileWrite   {
    fn set_name      (&self, name: &str)           -> Result<(),       &'static str>;
    fn set_size      (&self, size: usize)          -> Result<(),       &'static str>;
    fn set_timestamp (&self, timestamp: i64)       -> Result<(),       &'static str>;
    fn set_write     (&self, write: bool)          -> Result<(),       &'static str>;
    fn set_hidden    (&self, hidden: bool)         -> Result<(),       &'static str>;
}


// STRUCTS
//Volume which simply exists in RAM
pub struct MemoryVolume {
    pub offset: usize,
    pub size: usize,
}
impl VolumeRead for MemoryVolume {
    fn read  (&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
        if offset + buffer.len() > self.size {return Err("Memory Volume: Read out of bounds.")}
        for (i, byte) in buffer.iter_mut().enumerate() {unsafe {asm!(
            "MOV {reg:l}, [{src}]",
            "MOV [{dest}], {reg:l}",
            src  = in(reg) self.offset + offset + i,
            dest = in(reg) byte as *mut u8,
            reg  = out(reg) _,
        );}}
        Ok(())
    }
}
impl VolumeWrite for MemoryVolume {
    fn write (&self, offset: usize, buffer: &[u8])     -> Result<(), &'static str> {
        if offset + buffer.len() > self.size {return Err("Memory Volume: Read out of bounds.")}
        for (i, byte) in buffer.iter().enumerate() {unsafe {asm!(
            "MOV {reg:l}, [{src}]",
            "MOV [{dest}], {reg:l}",
            src  = in(reg) byte as *const u8,
            dest = in(reg) self.offset + offset + i,
            reg  = out(reg) _,
        );}}
        Ok(())
    }
}

//Volume which reads at an offset from another volume
pub struct VolumeFromVolume<'s, V:'s> {
    pub volume: &'s V,
    pub offset: usize,
    pub size:   usize,
}
impl<'s, RO:'s + VolumeRead>  VolumeRead  for VolumeFromVolume<'s, RO> {
    fn read  (&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
        if offset + buffer.len() > self.size {return Err("Volume on Volume: Read out of bounds.")}
        self.volume.read(self.offset + offset, buffer)
    }
}
impl<'s, WO:'s + VolumeWrite> VolumeWrite for VolumeFromVolume<'s, WO> {
    fn write (&self, offset: usize, buffer: &[u8])     -> Result<(), &'static str> {
        if offset + buffer.len() > self.size {return Err("Volume on Volume: Read out of bounds.")}
        self.volume.write(self.offset + offset, buffer)
    }
}
