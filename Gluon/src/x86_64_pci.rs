// GLUON: x86-64 PCI
// Structs and objects related to the handling of the PCI bus


// HEADER
//Imports
use crate::*;


// PCI DEVICES
//PCI Function Endpoint
pub struct PciEndpoint {
    bus: u32,
    device: u32,
    function: u32,
}
impl PciEndpoint {
    unsafe fn global_register(bus: u32, device: u32, function: u32, register: u32) -> Result<u32, &'static str> {
        //Bounds checking
        if bus      > 0xFF {return Err("PCI Register: Bus out of bounds.")}
        if device   > 0x1F {return Err("PCI Register: Device out of bounds.")}
        if function > 0x07 {return Err("PCI Register: Function out of bounds.")}
        if register > 0x3F {return Err("PCI Register: Register out of bounds.")}
        //Read register
        PORT_PCI_INDEX.write((1<<31)|(bus<<16)|(device<<11)|(function<<8)|(register<<2));
        Ok(PORT_PCI_DATA.read())
    }

    pub unsafe fn register(&self, register: u32) -> Result<u32, &'static str> {
        PciEndpoint::global_register(self.bus, self.device, self.function, register)
    }

    //Constructor
    pub unsafe fn new(bus: u32, device: u32, function: u32) -> Result<Self, &'static str> {
        //Bounds checking
        if bus      > 0xFF {return Err("PCI Device: Bus index out of bounds.")}
        if device   > 0x1F {return Err("PCI Device: Device index out of bounds.")}
        if function > 0x07 {return Err("PCI Device: Function index out of bounds.")}
        //Check function exists
        if (PciEndpoint::global_register(bus, device, function, 0x00)? & 0x0000FFFF) == 0xFFFF {return Err("PCI Device: No device found.")}
        //Return self
        Ok(Self {bus, device, function})
    }

    //Read Header
    pub unsafe fn vendor_id  (&self) -> u32 { self.register(0x00).unwrap() & 0x0000FFFF         }
    pub unsafe fn device_id  (&self) -> u32 {(self.register(0x00).unwrap() & 0xFFFF0000) >> 0x10}
    pub unsafe fn status     (&self) -> u32 {(self.register(0x01).unwrap() & 0xFFFF0000) >> 0x10}
    pub unsafe fn revision_id(&self) -> u32 { self.register(0x02).unwrap() & 0x000000FF         }
    pub unsafe fn prog_if    (&self) -> u32 {(self.register(0x02).unwrap() & 0x0000FF00) >> 0x08}
    pub unsafe fn subclass   (&self) -> u32 {(self.register(0x02).unwrap() & 0x00FF0000) >> 0x10}
    pub unsafe fn class_code (&self) -> u32 {(self.register(0x02).unwrap() & 0xFF000000) >> 0x18}
    pub unsafe fn chache_lz  (&self) -> u32 { self.register(0x03).unwrap() & 0x000000FF         }
    pub unsafe fn latency    (&self) -> u32 {(self.register(0x03).unwrap() & 0x0000FF00) >> 0x08}
    pub unsafe fn header_type(&self) -> u32 {(self.register(0x03).unwrap() & 0x007F0000) >> 0x10}
    pub unsafe fn bist       (&self) -> u32 {(self.register(0x03).unwrap() & 0xFF000000) >> 0x18}
}

//UHCI Endpoint
pub struct PciUhci {
    pub pci:        PciEndpoint,
    pub command:    PortGeneric<u16, ReadWriteAccess>,
    pub status:     PortGeneric<u16, ReadWriteAccess>,
    pub interrupt:  PortGeneric<u16, ReadWriteAccess>,
    pub frame_num:  PortGeneric<u16, ReadWriteAccess>,
    pub frame_base: PortGeneric<u32, ReadWriteAccess>,
    pub frame_mod:  PortGeneric<u8,  ReadWriteAccess>,
    pub sc_1:       PortGeneric<u16, ReadWriteAccess>,
    pub sc_2:       PortGeneric<u16, ReadWriteAccess>
}
impl PciUhci {
    pub unsafe fn new(pci: PciEndpoint) -> Result<Self, &'static str> {
        if !(pci.class_code() == 0x0C && pci.subclass() == 0x03 && pci.prog_if() == 0x00) {return Err("PCI UHCI: Device is not UHCI.")};
        let reg = pci.register(0x08)?;
        let baseport = if reg > 0xFFFF {return Err("PCI UHCI: Base port out of bounds.")} else {reg as u16};
        Ok(Self {
            pci,
            command:    PortGeneric::<u16, ReadWriteAccess>::new(baseport       ),
            status:     PortGeneric::<u16, ReadWriteAccess>::new(baseport + 0x02),
            interrupt:  PortGeneric::<u16, ReadWriteAccess>::new(baseport + 0x04),
            frame_num:  PortGeneric::<u16, ReadWriteAccess>::new(baseport + 0x06),
            frame_base: PortGeneric::<u32, ReadWriteAccess>::new(baseport + 0x08),
            frame_mod:  PortGeneric::<u8,  ReadWriteAccess>::new(baseport + 0x0C),
            sc_1:       PortGeneric::<u16, ReadWriteAccess>::new(baseport + 0x10),
            sc_2:       PortGeneric::<u16, ReadWriteAccess>::new(baseport + 0x12),
        })
    }
}
