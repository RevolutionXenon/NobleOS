// GLUON: PC Port Space
// Functions and objects related to the handling of the PC architecture's standard port-space layout


// HEADER
//Imports
use crate::x86_64::port::*;


// PORT SPACE
//Functions
pub fn io_wait() {unsafe {WAIT.write(0x00);}}

//Ports
pub static mut PIC1_COMMAND:  PortB = PortB(0x0020);
pub static mut PIC1_DATA:     PortB = PortB(0x0021);
pub static mut PIT_CHANNEL_1: PortB = PortB(0x0040);
pub static mut PIT_CHANNEL_2: PortB = PortB(0x0041);
pub static mut PIT_CHANNEL_3: PortB = PortB(0x0042);
pub static mut PIT_COMMAND:   PortB = PortB(0x0043);
pub static mut PS2_DATA:      PortB = PortB(0x0060);
pub static mut PS2_COMMAND:   PortB = PortB(0x0064);
pub static mut PS2_STATUS:    PortB = PortB(0x0064);
pub static mut WAIT:          PortB = PortB(0x0080);
pub static mut PIC2_COMMAND:  PortB = PortB(0x00A0);
pub static mut PIC2_DATA:     PortB = PortB(0x00A1);
pub static mut SERIAL_4:      PortB = PortB(0x02E8);
pub static mut SERIAL_2:      PortB = PortB(0x02F8);
pub static mut SERIAL_3:      PortB = PortB(0x03E8);
pub static mut SERIAL_1:      PortB = PortB(0x03F8);
pub static mut PCI_INDEX:     PortD = PortD(0x0CF8);
pub static mut PCI_DATA:      PortD = PortD(0x0CFC);
