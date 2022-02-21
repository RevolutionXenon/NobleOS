// GLUON: PERSONAL COMPUTER PERIPHERAL COMPONENT INTERCONNECT
// Structs and objects related to the handling of the PCI bus


// HEADER
//Imports
use crate::pc::ports::PCI_INDEX as INDEX_PORT;
use crate::pc::ports::PCI_DATA as DATA_PORT;
use crate::x86_64::port::*;


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
        INDEX_PORT.write((1<<31)|(bus<<16)|(device<<11)|(function<<8)|(register<<2));
        Ok(DATA_PORT.read())
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
    pub command:    PortW,
    pub status:     PortW,
    pub interrupt:  PortW,
    pub frame_num:  PortW,
    pub frame_base: PortD,
    pub frame_mod:  PortB,
    pub sc_1:       PortW,
    pub sc_2:       PortW,
}
impl PciUhci {
    pub unsafe fn new(pci: PciEndpoint) -> Result<Self, &'static str> {
        if !(pci.class_code() == 0x0C && pci.subclass() == 0x03 && pci.prog_if() == 0x00) {return Err("PCI UHCI: Device is not UHCI.")};
        let reg = pci.register(0x08)?;
        let baseport = if reg > 0xFFFF {return Err("PCI UHCI: Base port out of bounds.")} else {reg as u16};
        Ok(Self {
            pci,
            command:    PortW(baseport       ),
            status:     PortW(baseport + 0x02),
            interrupt:  PortW(baseport + 0x04),
            frame_num:  PortW(baseport + 0x06),
            frame_base: PortD(baseport + 0x08),
            frame_mod:  PortB(baseport + 0x0C),
            sc_1:       PortW(baseport + 0x10),
            sc_2:       PortW(baseport + 0x12),
        })
    }
}
