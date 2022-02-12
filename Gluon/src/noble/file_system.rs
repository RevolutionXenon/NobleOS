// GLUON: NOBLE FILE SYSTEM
// Structs and traits for handling file systems in a generic manner

// HEADER
//Imports
use core::arch::asm;
use core::ptr::addr_of;


// TRAITS
//Volume Read
pub trait VolumeRead  {
    fn read  (&self, offset: usize, buffer: &mut [u8]) -> Result<(), &'static str>;
}
//Volume Write
pub trait VolumeWrite {
    fn write (&self, offset: usize, buffer: &[u8])     -> Result<(), &'static str>;
}
//File Handle Read
pub trait FileRead    {
    fn get_name<'f>  (&self, buffer: &'f mut [u8]) -> Result<&'f str,  &'static str>;
    fn get_size      (&self)                       -> Result<usize,    &'static str>;
    fn get_timestamp (&self)                       -> Result<i64,      &'static str>;
    fn get_write     (&self)                       -> Result<bool,     &'static str>;
    fn get_hidden    (&self)                       -> Result<bool,     &'static str>;
}
//File Handle Write
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
        for i in 0..buffer.len() {unsafe {asm!(
            "MOV AL, [{src}]",
            "MOV [{dest}], AL",
            src  = in(reg) self.offset + offset + i,
            dest = in(reg) &mut buffer[i] as *mut _,
            lateout("rax") _,
        );}}
        Ok(())
    }
}
impl VolumeWrite for MemoryVolume {
    fn write (&self, offset: usize, buffer: &[u8])     -> Result<(), &'static str> {
        if offset + buffer.len() > self.size {return Err("Memory Volume: Read out of bounds.")}
        for i in 0..buffer.len() {unsafe {asm!(
            "MOV AL, [{src}]",
            "MOV [{dest}], AL",
            src  = in(reg) &buffer[i] as *const _,
            dest = in(reg) self.offset + offset + i,
            lateout("rax") _,
        );}}
        Ok(())
    }
}
