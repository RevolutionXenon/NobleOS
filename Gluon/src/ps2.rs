// GLUON: PS/2
// Structs and objects related to the handling of the PS/2 controller and devices

use x86_64::instructions::port::*;

// PS/2 CONTROLLER
//Ports
static mut PS2_DATA: PortGeneric::<u8, ReadWriteAccess> = PortGeneric::<u8, ReadWriteAccess>::new(0x60);
static mut PS2_COMMAND: PortGeneric::<u8, WriteOnlyAccess> = PortGeneric::<u8, WriteOnlyAccess>::new(0x64);
static mut PS2_STATUS: PortGeneric::<u8, ReadOnlyAccess> = PortGeneric::<u8, ReadOnlyAccess>::new(0x64);

pub struct Ps2Controller {}
impl Ps2Controller {
    pub unsafe fn read_memory(&self, address: u8) -> Result<u8, &'static str> {
        if address > 0x1F {return Err("PS/2 Controller: Memory read address out of bounds.")}
        PS2_COMMAND.write(address | 0x20);
        while PS2_STATUS.read() & 0x01 == 0 {}
        Ok(PS2_DATA.read())
    }
    pub unsafe fn write_memory(&self, address: u8, data: u8) -> Result<(), &'static str> {
        if address > 0x1F {return Err("PS/2 Controller: Memory write address out of bounds.")}
        PS2_COMMAND.write(address | 0x60);
        while PS2_STATUS.read() & 0x02 != 0 {}
        PS2_DATA.write(data);
        Ok(())
    }

    pub unsafe fn test_controller(&self) -> bool {PS2_COMMAND.write(0xAA); while PS2_STATUS.read() & 0x01 == 0 {} PS2_DATA.read() == 0x55}
    pub unsafe fn test_port_1    (&self) -> bool {PS2_COMMAND.write(0xAB); while PS2_STATUS.read() & 0x01 == 0 {} PS2_DATA.read() == 0x00}
    pub unsafe fn test_port_2    (&self) -> bool {PS2_COMMAND.write(0xA9); while PS2_STATUS.read() & 0x01 == 0 {} PS2_DATA.read() == 0x00}

    pub unsafe fn enable_port_1 (&self) {PS2_COMMAND.write(0xAE)}
    pub unsafe fn disable_port_1(&self) {PS2_COMMAND.write(0xAD)}
    pub unsafe fn enable_port_2 (&self) {PS2_COMMAND.write(0xA8)}
    pub unsafe fn disable_port_2(&self) {PS2_COMMAND.write(0xA7)}
}
