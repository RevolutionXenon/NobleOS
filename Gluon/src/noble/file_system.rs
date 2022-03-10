// GLUON: NOBLE FILE SYSTEM
// Structs and traits for handling file systems in a generic manner


// HEADER
//Imports
use crate::noble::return_code::ReturnCode;
use core::{arch::asm, convert::TryInto};


// MACROS
#[macro_export]
macro_rules!return_if_partial {
    ($complete: expr, $call: expr) => {
        match ($call)? {
            None    => {},
            Some(n) => {return Ok($complete + n)}
        }
    };
}


// TRAITS
//Volume Read
pub trait VolumeRead  {
    fn read     (&self, offset: u64, buffer: &mut [u8]) -> Result<u64, ReturnCode>;
    fn read_all (&self, offset: u64, buffer: &mut [u8]) -> Result<(),  ReturnCode> {
        let mut i = 0;
        loop {
            i = self.read(offset, &mut buffer[i..])? as usize;
            if i == buffer.len() {break}
        }
        Ok(())
    }
    fn read_check (&self, offset: u64, buffer: &mut [u8]) -> Result<Option<u64>, ReturnCode> {
        let i = self.read(offset, buffer)?;
        if i == buffer.len() as u64 {Ok(None)}
        else {Ok(Some(i))}
    }
}
impl<'a, T: 'a+VolumeRead>  VolumeRead  for &'a T {
    fn read  (&self, offset: u64, buffer: &mut [u8]) -> Result<u64, ReturnCode> {
        (**self).read(offset, buffer)
    }
}

//Volume Write
pub trait VolumeWrite {
    fn write     (&self, offset: u64, buffer: &[u8]) -> Result<u64, ReturnCode>;
    fn write_all (&self, offset: u64, buffer: &[u8]) -> Result<(), ReturnCode> {
        let mut i = 0;
        loop {
            i = self.write(offset, &buffer[i..])? as usize;
            if i == buffer.len() {break}
        }
        Ok(())
    }
    fn write_check (&self, offset: u64, buffer: &[u8]) -> Result<Option<u64>, ReturnCode> {
        let i = self.write(offset, buffer)?;
        if i == buffer.len() as u64 {Ok(None)}
        else {Ok(Some(i))}
    }
}
impl<'a, T: 'a+VolumeWrite> VolumeWrite for &'a T {
    fn write  (&self, offset: u64, buffer: &[u8])    -> Result<u64, ReturnCode> {
        (**self).write(offset, buffer)
    }
}

//Read+Write
pub trait VolumeReadWrite: VolumeRead+VolumeWrite {}
impl<T: VolumeRead+VolumeWrite> VolumeReadWrite for T {}
impl<'a> VolumeRead  for &'a dyn VolumeReadWrite {
    fn read     (&self, offset: u64, buffer: &mut [u8]) -> Result<u64, ReturnCode> {
        (**self).read(offset, buffer)
    }
}
impl<'a> VolumeWrite for &'a dyn VolumeReadWrite {
    fn write    (&self, offset: u64, buffer: &[u8])     -> Result<u64, ReturnCode> {
        (**self).write(offset, buffer)
    }
}


//File System
pub trait FileSystem  {
    //Read and write files
    fn read      (&self, id: OpenFileID, offset: u64, buffer: &mut [u8])             -> Result<u64,         ReturnCode>;
    fn write     (&self, id: OpenFileID, offset: u64, buffer: &[u8])                 -> Result<u64,         ReturnCode>;
    //Open and close files
    fn open      (&self, id: FileID)                                                 -> Result<OpenFileID,  ReturnCode>;
    fn close     (&self, id: OpenFileID)                                             -> Result<(),          ReturnCode>;
    //Create and delete files
    fn create    (&self, directory_id: OpenFileID, name: &str, size: u64, dir: bool) -> Result<OpenFileID,  ReturnCode>;
    fn delete    (&self, directory_id: OpenFileID, name: &str)                       -> Result<(),          ReturnCode>;
    //Traverse directories
    fn root      (&self)                                                             -> Result<FileID,      ReturnCode>;
    fn dir_first (&self, directory_id: OpenFileID)                                   -> Result<Option<u64>, ReturnCode>;
    fn dir_next  (&self, directory_id: OpenFileID, index: u64)                       -> Result<Option<u64>, ReturnCode>;
    fn dir_name  (&self, directory_id: OpenFileID, name: &str)                       -> Result<Option<u64>, ReturnCode>;
    //File properties
    fn get_id    (&self, directory_id: OpenFileID, index: u64)                       -> Result<FileID,      ReturnCode>;
    fn get_dir   (&self, directory_id: OpenFileID, index: u64)                       -> Result<bool,        ReturnCode>;
    fn get_size  (&self, directory_id: OpenFileID, index: u64)                       -> Result<u64,         ReturnCode>;
    fn set_size  (&self, directory_id: OpenFileID, index: u64, size: u64)            -> Result<(),          ReturnCode>;
    fn get_name  (&self, directory_id: OpenFileID, index: u64, buffer: &mut[u8])     -> Result<&str,        ReturnCode>;
    fn set_name  (&self, directory_id: OpenFileID, index: u64, name: &str)           -> Result<(),          ReturnCode>;
}


// STRUCTS
#[derive(Clone, Copy, Debug)] pub struct FileID(pub u64);
#[derive(Clone, Copy, Debug)] pub struct OpenFileID(pub u64);

//File Handle
pub struct FileShortcut<'s> {
    pub fs: &'s dyn FileSystem,
    pub id: OpenFileID,
}
impl<'s> VolumeRead for FileShortcut<'s> {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<u64, ReturnCode> {
        self.fs.read(self.id, offset, buffer)
    }
}
impl<'s> VolumeWrite for FileShortcut<'s> {
    fn write(&self, offset: u64, buffer: &[u8]) -> Result<u64, ReturnCode> {
        self.fs.write(self.id, offset, buffer)
    }
}
impl<'s> Drop for FileShortcut<'s> {
    fn drop(&mut self) {
        self.fs.close(self.id).unwrap();
    }
}

//Volume which simply exists in RAM
pub struct MemoryVolume {
    pub offset: usize,
    pub size: usize,
}
impl VolumeRead for MemoryVolume {
    fn read  (&self, offset: u64, buffer: &mut [u8]) -> Result<u64, ReturnCode> {
        let offset: usize = offset.try_into().map_err(|_| ReturnCode::MemoryOutOfBounds)?;
        if offset + buffer.len() > self.size {return Err(ReturnCode::EndOfVolume)}
        for (i, byte) in buffer.iter_mut().enumerate() {unsafe {asm!(
            "MOV {reg:l}, [{src}]",
            "MOV [{dest}], {reg:l}",
            src  = in(reg) self.offset + offset + i,
            dest = in(reg) byte as *mut u8,
            reg  = out(reg) _,
        );}}
        Ok(buffer.len() as u64)
    }
}
impl VolumeWrite for MemoryVolume {
    fn write (&self, offset: u64, buffer: &[u8])     -> Result<u64, ReturnCode> {
        let offset: usize = offset.try_into().map_err(|_| ReturnCode::MemoryOutOfBounds)?;
        if offset + buffer.len() > self.size {return Err(ReturnCode::EndOfVolume)}
        for (i, byte) in buffer.iter().enumerate() {unsafe {asm!(
            "MOV {reg:l}, [{src}]",
            "MOV [{dest}], {reg:l}",
            src  = in(reg) byte as *const u8,
            dest = in(reg) self.offset + offset + i,
            reg  = out(reg) _,
        );}}
        Ok(buffer.len() as u64)
    }
}

//Volume which reads at an offset from another volume
pub struct VolumeFromVolume<'s, V:'s> {
    pub volume: &'s V,
    pub offset: u64,
    pub size:   u64,
}
impl<'s, RO:'s + VolumeRead>  VolumeRead  for VolumeFromVolume<'s, RO> {
    fn read  (&self, offset: u64, buffer: &mut [u8]) -> Result<u64, ReturnCode> {
        if offset + buffer.len() as u64 > self.size {return Err(ReturnCode::EndOfVolume)}
        self.volume.read(self.offset + offset, buffer)
    }
}
impl<'s, WO:'s + VolumeWrite> VolumeWrite for VolumeFromVolume<'s, WO> {
    fn write (&self, offset: u64, buffer: &[u8])     -> Result<u64, ReturnCode> {
        if offset + buffer.len() as u64 > self.size {return Err(ReturnCode::EndOfVolume)}
        self.volume.write(self.offset + offset, buffer)
    }
}
