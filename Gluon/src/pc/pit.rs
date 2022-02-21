// GLUON: PROGRAMMABLE INTERVAL TIMER
// Consts, Functions, and Enums related to the handling of the 8253 and 8254 Programmable Interval Timer


// HEADER
//Imports
use crate::pc::ports::PIT_CHANNEL_1 as CHANNEL_1;
use crate::pc::ports::PIT_CHANNEL_2 as CHANNEL_2;
use crate::pc::ports::PIT_CHANNEL_3 as CHANNEL_3;
use crate::pc::ports::PIT_COMMAND as COMMAND;
use crate::x86_64::port::*;

//Constants
pub const PIT_FREQUENCY: usize = 1193182;


// PROGRAMMABLE INTERVAL TIMER
//Send Command
pub unsafe fn send_command(channel: Channel, access_mode: AccessMode, operating_mode: OperatingMode, binary_mode: BinaryMode) {
    let mut byte: u8 = 0;
    byte |= (channel        as u8) << 6;
    byte |= (access_mode    as u8) << 4;
    byte |= (operating_mode as u8) << 1;
    byte |=  binary_mode    as u8;
    COMMAND.write(byte);
}

//Set Reload
pub unsafe fn set_reload_full(channel: Channel, value: u16) {
    let bytes = value.to_le_bytes();
    match channel {
        Channel::C1 => {CHANNEL_1.write(bytes[0]); CHANNEL_1.write(bytes[1]);},
        Channel::C2 => {CHANNEL_2.write(bytes[0]); CHANNEL_2.write(bytes[1]);},
        Channel::C3 => {CHANNEL_3.write(bytes[0]); CHANNEL_3.write(bytes[1]);},
    }
}
pub unsafe fn set_reload_half(channel: Channel, value: u8) {
    match channel {
        Channel::C1 => {CHANNEL_1.write(value);},
        Channel::C2 => {CHANNEL_2.write(value);},
        Channel::C3 => {CHANNEL_3.write(value);},
    }
}

//Read Count
pub unsafe fn read_count_full(channel: Channel) -> u16 {
    send_command(channel, AccessMode::LatchCount, OperatingMode::TerminalCount, BinaryMode::Binary);
    let mut bytes = [0u8;2];
    match channel {
        Channel::C1 => {bytes[0] = CHANNEL_1.read(); bytes[1] = CHANNEL_1.read();},
        Channel::C2 => {bytes[0] = CHANNEL_2.read(); bytes[1] = CHANNEL_2.read();},
        Channel::C3 => {bytes[0] = CHANNEL_3.read(); bytes[1] = CHANNEL_3.read();},
    }
    u16::from_le_bytes(bytes)
}
pub unsafe fn read_count_half(channel: Channel) -> u8 {
    match channel {
        Channel::C1 => CHANNEL_1.read(),
        Channel::C2 => CHANNEL_2.read(),
        Channel::C3 => CHANNEL_3.read(),
    }
}

//Channel Select
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Channel {
    C1 = 0x00,
    C2 = 0x01,
    C3 = 0x02,
}

//Access Mode
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AccessMode {
    LatchCount = 0x00,
    LowByte    = 0x01,
    HighBite   = 0x02,
    Full       = 0x03,
}

//Operating Mode
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OperatingMode {
    TerminalCount  = 0x00,
    OneShot        = 0x01,
    RateGenerator  = 0x02,
    SquareWave     = 0x03,
    SoftwareStrobe = 0x04,
    HardwareStrobe = 0x05,
    RateGenerator2 = 0x06,
    SquareWave2    = 0x07,
}

//Binary Mode
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BinaryMode {
    Binary = 0x0,
    BCD    = 0x1,
}
