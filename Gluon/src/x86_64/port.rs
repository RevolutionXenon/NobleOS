// GLUON: x86-64 PORT
// Structs, functions, and traits related to the handling of ports


// HEADER
//Imports
use crate::x86_64::instructions::in_b;
use crate::x86_64::instructions::in_w;
use crate::x86_64::instructions::in_d;
use crate::x86_64::instructions::out_b;
use crate::x86_64::instructions::out_w;
use crate::x86_64::instructions::out_d;


// PORT
//Trait for Ports
pub trait Port {
    type size;
    fn read  (&self)     -> Self::size;
    fn write (&self, value: Self::size);
}

//8-bit Port
pub struct PortB(pub u16);
impl Port for PortB {
    type size = u8;
    fn read  (&self)     -> Self::size  {in_b(self.0)}
    fn write (&self, value: Self::size) {out_b(self.0, value)}
}

//16-bit Port
pub struct PortW(pub u16);
impl Port for PortW {
    type size = u16;
    fn read  (&self)     -> Self::size  {in_w(self.0)}
    fn write (&self, value: Self::size) {out_w(self.0, value)}
}

//32-bit Port
pub struct PortD(pub u16);
impl Port for PortD {
    type size = u32;
    fn read  (&self)     -> Self::size  {in_d(self.0)}
    fn write (&self, value: Self::size) {out_d(self.0, value)}
}
