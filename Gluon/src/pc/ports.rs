// GLUON: PC Port Space
// Functions and objects related to the handling of the PC architecture's standard port-space layout


// HEADER
//Imports
use ::x86_64::instructions::port::*;


// PORT SPACE
//Functions
pub fn io_wait() {
    unsafe {PORT_WAIT.write(0x00);}
}

//Ports
pub static mut PORT_PIC1_COMMAND:   PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x0020);
pub static mut PORT_PIC1_DATA:      PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0021);
pub static mut PORT_PIT_CHANNEL_1:  PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0040);
pub static mut PORT_PIT_CHANNEL_2:  PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0041);
pub static mut PORT_PIT_CHANNEL_3:  PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0042);
pub static mut PORT_PIT_COMMAND:    PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x0043);
pub static mut PORT_PS2_DATA:       PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x0060);
pub static mut PORT_PS2_COMMAND:    PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x0064);
pub static mut PORT_PS2_STATUS:     PortGeneric<u8,  ReadOnlyAccess > = PortGeneric::<u8,  ReadOnlyAccess >::new(0x0064);
pub static mut PORT_WAIT:           PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x0080);
pub static mut PORT_PIC2_COMMAND:   PortGeneric<u8,  WriteOnlyAccess> = PortGeneric::<u8,  WriteOnlyAccess>::new(0x00A0);
pub static mut PORT_PIC2_DATA:      PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x00A1);
pub static mut PORT_SERIAL_4:       PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x02E8);
pub static mut PORT_SERIAL_2:       PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x02F8);
pub static mut PORT_SERIAL_3:       PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x03E8);
pub static mut PORT_SERIAL_1:       PortGeneric<u8,  ReadWriteAccess> = PortGeneric::<u8,  ReadWriteAccess>::new(0x03F8);
pub static mut PORT_PCI_INDEX:      PortGeneric<u32, ReadWriteAccess> = PortGeneric::<u32, ReadWriteAccess>::new(0x0CF8);
pub static mut PORT_PCI_DATA:       PortGeneric<u32, ReadWriteAccess> = PortGeneric::<u32, ReadWriteAccess>::new(0x0CFC);
