// GLUON: x86-64 PS/2
// Structs and objects related to the handling of the PS/2 controller and devices


// HEADER
//Imports
use crate::noble::input_events::*;
use crate::pc::ports::PORT_PS2_COMMAND as COMMAND_PORT;
use crate::pc::ports::PORT_PS2_DATA as DATA_PORT;
use crate::pc::ports::PORT_PS2_STATUS as STATUS_PORT;


// PS/2 CONTROLLER
//Generic Controller Functions
pub unsafe fn read_memory(address: u8) -> Result<u8, &'static str> {
    if address > 0x1F {return Err("PS/2 Controller: Memory read address out of bounds.")}
    COMMAND_PORT.write(address | 0x20);
    while STATUS_PORT.read() & 0x01 == 0 {}
    Ok(DATA_PORT.read())
}
pub unsafe fn write_memory(address: u8, data: u8) -> Result<(), &'static str> {
    if address > 0x1F {return Err("PS/2 Controller: Memory write address out of bounds.")}
    COMMAND_PORT.write(address | 0x60);
    while STATUS_PORT.read() & 0x02 != 0 {}
    DATA_PORT.write(data);
    Ok(())
}

//Test Functions
pub unsafe fn test_controller() -> bool {COMMAND_PORT.write(0xAA); wait_for_output(); read_output() == 0x55}
pub unsafe fn test_port_1()     -> bool {COMMAND_PORT.write(0xAB); wait_for_output(); read_output() == 0x00}
pub unsafe fn test_port_2()     -> bool {COMMAND_PORT.write(0xA9); wait_for_output(); read_output() == 0x00}

//Port Functions
pub unsafe fn enable_port1()  {COMMAND_PORT.write(0xAE)}
pub unsafe fn disable_port1() {COMMAND_PORT.write(0xAD)}
pub unsafe fn enable_port2()  {COMMAND_PORT.write(0xA8)}
pub unsafe fn disable_port2() {COMMAND_PORT.write(0xA7)}

//Interrupt Functions
pub unsafe fn enable_int_port1()  {write_memory(0x00, read_memory(0x00).unwrap() | 0x01).unwrap();}
pub unsafe fn disable_int_port1() {write_memory(0x00, read_memory(0x00).unwrap() & 0xFE).unwrap();}
pub unsafe fn enable_int_port2()  {write_memory(0x00, read_memory(0x00).unwrap() | 0x02).unwrap();}
pub unsafe fn disable_int_port2() {write_memory(0x00, read_memory(0x00).unwrap() & 0xFD).unwrap();}

//Read Functions
pub unsafe fn read_output() -> u8 {DATA_PORT.read()}
pub unsafe fn read_status() -> u8 {STATUS_PORT.read()}

//Poll Functions
pub unsafe fn poll_output_buffer_status() -> bool {read_status() & 0x01 > 0}
pub unsafe fn poll_input_buffer_status() ->  bool {read_status() & 0x02 > 0}

//Wait Functions
pub unsafe fn wait_for_output() {while !poll_output_buffer_status() {}}
pub unsafe fn wait_for_input() {while poll_input_buffer_status() {}}

//Flush Function
pub unsafe fn flush_output() {while poll_output_buffer_status() {read_output();}}


// PS/2 KEYBOARD
//Check Response Function
pub unsafe fn keyboard_check_response() -> Result<(), &'static str> {
    match DATA_PORT.read() {
        0x00 => Err("BUFFER OVERRUN"),
        0xFA => Ok(()),
        0xFE => Err("RESEND"),
        0xFF => Err("KEY DETECTION ERROR"),
        //_ => Err("UNKNOWN ERROR"),
        byte => panic!("UNKNOWN ERROR {}", byte),
    }
}

//Scancode Set Functions
pub unsafe fn keyboard_get_scancode_set() -> Result<u8, &'static str> {
    flush_output();
    let retries = 4;
    for i in 0..retries+1 {
        wait_for_input();
        DATA_PORT.write(0xF0);
        wait_for_output();
        match DATA_PORT.read() {
            0x00 => return Err("PS/2 Keyboard: Get scancode buffer overrun after sending command byte."),
            0xFA => break,
            0xFE => {},
            0xFF => return Err("PS/2 Keyboard: Get scancode key detection error after sending command byte."),
            _ => return Err("PS/2 Keyboard: Get scancode unknown error after sending command byte."),
        }
        if i == retries {return Err("PS/2 Keyboard: Get scancode ran out of retries after sending command byte.")}
    }
    flush_output();
    wait_for_input();
    DATA_PORT.write(0);
    wait_for_output();
    match DATA_PORT.read() {
        0xFA => {},
        _ => return Err("PS/2 Keyboard: Get scancode recieved error after sending second byte.")
    }
    wait_for_output();
    Ok(DATA_PORT.read())
}
pub unsafe fn keyboard_set_scancode_set(set: u8) -> Result<(), &'static str> {
    if set > 3 || set == 0 {return Err("PS/2 Keyboard: Set scancode provided with invalid value for set.")}
    let retries = 4;
    for i in 0..retries+1 {
        wait_for_input();
        DATA_PORT.write(0xF0);
        wait_for_output();
        match DATA_PORT.read() {
            0x00 => return Err("PS/2 Keyboard: Set scancode buffer overrun after sending command byte."),
            0xFA => break,
            0xFE => {},
            0xFF => return Err("PS/2 Keyboard: Set scancode key detection error after sending command byte."),
            _ => return Err("PS/2 Keyboard: Set scancode unknown error after sending command byte."),
        }
        if i == retries {return Err("PS/2 Keyboard: Set scancode ran out of retries after sending command byte.")}
    }
    wait_for_input();
    DATA_PORT.write(set);
    wait_for_output();
    match DATA_PORT.read() {
        0xFA => Ok(()),
        _ => Err("PS/2 Keyboard: Set scancode recieved error after sending set byte.")
    }
}

//Scan Functions
pub unsafe fn keyboard_enable_scan()  -> Result<(), &'static str> {
    wait_for_input();
    DATA_PORT.write(0xF4);
    wait_for_output();
    keyboard_check_response()
}
pub unsafe fn keyboard_disable_scan() -> Result<(), &'static str> {
    wait_for_input();
    DATA_PORT.write(0xF5);
    wait_for_output();
    keyboard_check_response()
}


// PS/2 Scancode Translation Return Value
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum Ps2Scan {
    Finish(InputEvent),
    Continue,
}

// Scancode Set 1 to input event translation
pub fn scancodes_1(scancodes: &[u8], device_id: u16) -> Result<Ps2Scan, &'static str> {
    match scancodes.len() {
        0x01 => match scancodes[0] {
            0x01 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEscape       as u16, event_data: PressType::Press as i16})),
            0x02 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key1            as u16, event_data: PressType::Press as i16})),
            0x03 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key2            as u16, event_data: PressType::Press as i16})),
            0x04 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key3            as u16, event_data: PressType::Press as i16})),
            0x05 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key4            as u16, event_data: PressType::Press as i16})),
            0x06 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key5            as u16, event_data: PressType::Press as i16})),
            0x07 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key6            as u16, event_data: PressType::Press as i16})),
            0x08 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key7            as u16, event_data: PressType::Press as i16})),
            0x09 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key8            as u16, event_data: PressType::Press as i16})),
            0x0B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key0            as u16, event_data: PressType::Press as i16})),
            0x0C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDash         as u16, event_data: PressType::Press as i16})),
            0x0D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEqual        as u16, event_data: PressType::Press as i16})),
            0x0E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyBackspace    as u16, event_data: PressType::Press as i16})),
            0x0F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyTab          as u16, event_data: PressType::Press as i16})),
            0x10 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyQ            as u16, event_data: PressType::Press as i16})),
            0x11 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyW            as u16, event_data: PressType::Press as i16})),
            0x12 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyE            as u16, event_data: PressType::Press as i16})),
            0x13 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyR            as u16, event_data: PressType::Press as i16})),
            0x14 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyT            as u16, event_data: PressType::Press as i16})),
            0x15 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyY            as u16, event_data: PressType::Press as i16})),
            0x16 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyU            as u16, event_data: PressType::Press as i16})),
            0x17 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyI            as u16, event_data: PressType::Press as i16})),
            0x18 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyO            as u16, event_data: PressType::Press as i16})),
            0x19 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyP            as u16, event_data: PressType::Press as i16})),
            0x1A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyOpenBracket  as u16, event_data: PressType::Press as i16})),
            0x1B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyCloseBracket as u16, event_data: PressType::Press as i16})),
            0x1C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEnter        as u16, event_data: PressType::Press as i16})),
            0x1D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftControl  as u16, event_data: PressType::Press as i16})),
            0x1E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyA            as u16, event_data: PressType::Press as i16})),
            0x1F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyS            as u16, event_data: PressType::Press as i16})),
            0x20 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyD            as u16, event_data: PressType::Press as i16})),
            0x21 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF            as u16, event_data: PressType::Press as i16})),
            0x22 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyG            as u16, event_data: PressType::Press as i16})),
            0x23 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyH            as u16, event_data: PressType::Press as i16})),
            0x24 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyJ            as u16, event_data: PressType::Press as i16})),
            0x25 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyK            as u16, event_data: PressType::Press as i16})),
            0x26 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyL            as u16, event_data: PressType::Press as i16})),
            0x27 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySemicolon    as u16, event_data: PressType::Press as i16})),
            0x28 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyQuote        as u16, event_data: PressType::Press as i16})),
            0x29 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyGrave        as u16, event_data: PressType::Press as i16})),
            0x2A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftShift    as u16, event_data: PressType::Press as i16})),
            0x2B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyBackSlash    as u16, event_data: PressType::Press as i16})),
            0x2C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyZ            as u16, event_data: PressType::Press as i16})),
            0x2D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyX            as u16, event_data: PressType::Press as i16})),
            0x2E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyC            as u16, event_data: PressType::Press as i16})),
            0x2F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyV            as u16, event_data: PressType::Press as i16})),
            0x30 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyB            as u16, event_data: PressType::Press as i16})),
            0x31 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyN            as u16, event_data: PressType::Press as i16})),
            0x32 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyM            as u16, event_data: PressType::Press as i16})),
            0x33 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyComma        as u16, event_data: PressType::Press as i16})),
            0x34 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPeriod       as u16, event_data: PressType::Press as i16})),
            0x35 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyForwardSlash as u16, event_data: PressType::Press as i16})),
            0x36 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightShift   as u16, event_data: PressType::Press as i16})),
            0x37 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumMultiply     as u16, event_data: PressType::Press as i16})),
            0x38 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftAlt      as u16, event_data: PressType::Press as i16})),
            0x39 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySpace        as u16, event_data: PressType::Press as i16})),
            0x3A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyCapsLock     as u16, event_data: PressType::Press as i16})),
            0x3B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF01          as u16, event_data: PressType::Press as i16})),
            0x3C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF02          as u16, event_data: PressType::Press as i16})),
            0x3D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF03          as u16, event_data: PressType::Press as i16})),
            0x3E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF04          as u16, event_data: PressType::Press as i16})),
            0x3F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF05          as u16, event_data: PressType::Press as i16})),
            0x40 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF06          as u16, event_data: PressType::Press as i16})),
            0x41 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF07          as u16, event_data: PressType::Press as i16})),
            0x42 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF08          as u16, event_data: PressType::Press as i16})),
            0x43 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF09          as u16, event_data: PressType::Press as i16})),
            0x44 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF10          as u16, event_data: PressType::Press as i16})),
            0x45 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumLock         as u16, event_data: PressType::Press as i16})),
            0x46 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyScrollLock   as u16, event_data: PressType::Press as i16})),
            0x47 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num7            as u16, event_data: PressType::Press as i16})),
            0x48 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num8            as u16, event_data: PressType::Press as i16})),
            0x49 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num9            as u16, event_data: PressType::Press as i16})),
            0x4A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumMinus        as u16, event_data: PressType::Press as i16})),
            0x4B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num4            as u16, event_data: PressType::Press as i16})),
            0x4C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num5            as u16, event_data: PressType::Press as i16})),
            0x4D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num6            as u16, event_data: PressType::Press as i16})),
            0x4E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumPlus         as u16, event_data: PressType::Press as i16})),
            0x4F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num1            as u16, event_data: PressType::Press as i16})),
            0x50 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num2            as u16, event_data: PressType::Press as i16})),
            0x51 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num3            as u16, event_data: PressType::Press as i16})),
            0x52 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num0            as u16, event_data: PressType::Press as i16})),
            0x53 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumPeriod       as u16, event_data: PressType::Press as i16})),
            0x57 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF11          as u16, event_data: PressType::Press as i16})),
            0x58 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF12          as u16, event_data: PressType::Press as i16})),
            0x81 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEscape       as u16, event_data: PressType::Unpress as i16})),
            0x82 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key1            as u16, event_data: PressType::Unpress as i16})),
            0x83 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key2            as u16, event_data: PressType::Unpress as i16})),
            0x84 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key3            as u16, event_data: PressType::Unpress as i16})),
            0x85 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key4            as u16, event_data: PressType::Unpress as i16})),
            0x86 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key5            as u16, event_data: PressType::Unpress as i16})),
            0x87 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key6            as u16, event_data: PressType::Unpress as i16})),
            0x88 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key7            as u16, event_data: PressType::Unpress as i16})),
            0x89 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key8            as u16, event_data: PressType::Unpress as i16})),
            0x8A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key9            as u16, event_data: PressType::Unpress as i16})),
            0x8B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key0            as u16, event_data: PressType::Unpress as i16})),
            0x8C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDash         as u16, event_data: PressType::Unpress as i16})),
            0x8D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEqual        as u16, event_data: PressType::Unpress as i16})),
            0x8E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyBackspace    as u16, event_data: PressType::Unpress as i16})),
            0x8F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyTab          as u16, event_data: PressType::Unpress as i16})),
            0x90 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyQ            as u16, event_data: PressType::Unpress as i16})),
            0x91 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyW            as u16, event_data: PressType::Unpress as i16})),
            0x92 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyE            as u16, event_data: PressType::Unpress as i16})),
            0x93 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyR            as u16, event_data: PressType::Unpress as i16})),
            0x94 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyT            as u16, event_data: PressType::Unpress as i16})),
            0x95 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyY            as u16, event_data: PressType::Unpress as i16})),
            0x96 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyU            as u16, event_data: PressType::Unpress as i16})),
            0x97 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyI            as u16, event_data: PressType::Unpress as i16})),
            0x98 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyO            as u16, event_data: PressType::Unpress as i16})),
            0x99 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyP            as u16, event_data: PressType::Unpress as i16})),
            0x9A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyOpenBracket  as u16, event_data: PressType::Unpress as i16})),
            0x9B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyCloseBracket as u16, event_data: PressType::Unpress as i16})),
            0x9C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEnter        as u16, event_data: PressType::Unpress as i16})),
            0x9D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftControl  as u16, event_data: PressType::Unpress as i16})),
            0x9E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyA            as u16, event_data: PressType::Unpress as i16})),
            0x9F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyS            as u16, event_data: PressType::Unpress as i16})),
            0xA0 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyD            as u16, event_data: PressType::Unpress as i16})),
            0xA1 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF            as u16, event_data: PressType::Unpress as i16})),
            0xA2 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyG            as u16, event_data: PressType::Unpress as i16})),
            0xA3 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyH            as u16, event_data: PressType::Unpress as i16})),
            0xA4 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyJ            as u16, event_data: PressType::Unpress as i16})),
            0xA5 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyK            as u16, event_data: PressType::Unpress as i16})),
            0xA6 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyL            as u16, event_data: PressType::Unpress as i16})),
            0xA7 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySemicolon    as u16, event_data: PressType::Unpress as i16})),
            0xA8 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyQuote        as u16, event_data: PressType::Unpress as i16})),
            0xA9 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyGrave        as u16, event_data: PressType::Unpress as i16})),
            0xAA => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftShift    as u16, event_data: PressType::Unpress as i16})),
            0xAB => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyBackSlash    as u16, event_data: PressType::Unpress as i16})),
            0xAC => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyZ            as u16, event_data: PressType::Unpress as i16})),
            0xAD => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyX            as u16, event_data: PressType::Unpress as i16})),
            0xAE => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyC            as u16, event_data: PressType::Unpress as i16})),
            0xAF => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyV            as u16, event_data: PressType::Unpress as i16})),
            0xB0 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyB            as u16, event_data: PressType::Unpress as i16})),
            0xB1 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyN            as u16, event_data: PressType::Unpress as i16})),
            0xB2 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyM            as u16, event_data: PressType::Unpress as i16})),
            0xB3 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyComma        as u16, event_data: PressType::Unpress as i16})),
            0xB4 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPeriod       as u16, event_data: PressType::Unpress as i16})),
            0xB5 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyForwardSlash as u16, event_data: PressType::Unpress as i16})),
            0xB6 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightShift   as u16, event_data: PressType::Unpress as i16})),
            0xB7 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumMultiply     as u16, event_data: PressType::Unpress as i16})),
            0xB8 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftAlt      as u16, event_data: PressType::Unpress as i16})),
            0xB9 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySpace        as u16, event_data: PressType::Unpress as i16})),
            0xBA => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyCapsLock     as u16, event_data: PressType::Unpress as i16})),
            0xBB => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF01          as u16, event_data: PressType::Unpress as i16})),
            0xBC => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF02          as u16, event_data: PressType::Unpress as i16})),
            0xBD => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF03          as u16, event_data: PressType::Unpress as i16})),
            0xBE => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF04          as u16, event_data: PressType::Unpress as i16})),
            0xBF => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF05          as u16, event_data: PressType::Unpress as i16})),
            0xC0 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF06          as u16, event_data: PressType::Unpress as i16})),
            0xC1 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF07          as u16, event_data: PressType::Unpress as i16})),
            0xC2 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF08          as u16, event_data: PressType::Unpress as i16})),
            0xC3 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF09          as u16, event_data: PressType::Unpress as i16})),
            0xC4 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF10          as u16, event_data: PressType::Unpress as i16})),
            0xC5 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumLock         as u16, event_data: PressType::Unpress as i16})),
            0xC6 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyScrollLock   as u16, event_data: PressType::Unpress as i16})),
            0xC7 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num7            as u16, event_data: PressType::Unpress as i16})),
            0xC8 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num8            as u16, event_data: PressType::Unpress as i16})),
            0xC9 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num9            as u16, event_data: PressType::Unpress as i16})),
            0xCA => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumMinus        as u16, event_data: PressType::Unpress as i16})),
            0xCB => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num4            as u16, event_data: PressType::Unpress as i16})),
            0xCC => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num5            as u16, event_data: PressType::Unpress as i16})),
            0xCD => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num6            as u16, event_data: PressType::Unpress as i16})),
            0xCE => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumPlus         as u16, event_data: PressType::Unpress as i16})),
            0xCF => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num1            as u16, event_data: PressType::Unpress as i16})),
            0xD0 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num2            as u16, event_data: PressType::Unpress as i16})),
            0xD1 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num3            as u16, event_data: PressType::Unpress as i16})),
            0xD2 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num0            as u16, event_data: PressType::Unpress as i16})),
            0xD3 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumPeriod       as u16, event_data: PressType::Unpress as i16})),
            0xD7 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF11          as u16, event_data: PressType::Unpress as i16})),
            0xD8 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF12          as u16, event_data: PressType::Unpress as i16})),
            0xE0 => Ok(Ps2Scan::Continue),
            _ => Err("PS2 Scancode Set 1: Unrecognized scancode [Invalid].")
        },
        0x02 => match scancodes[0] {
            0xE0 => match scancodes[1] {
                0x10 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaPreviousTrack      as u16, event_data: PressType::Press as i16})),
                0x19 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaNextTrack          as u16, event_data: PressType::Press as i16})),
                0x1C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumEnter                as u16, event_data: PressType::Press as i16})),
                0x1D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightControl         as u16, event_data: PressType::Press as i16})),
                0x20 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaMute               as u16, event_data: PressType::Press as i16})),
                0x21 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaCalculator         as u16, event_data: PressType::Press as i16})),
                0x22 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaPlayPause          as u16, event_data: PressType::Press as i16})),
                0x24 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaStop               as u16, event_data: PressType::Press as i16})),
                0x2E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaVolumeDown         as u16, event_data: PressType::Press as i16})),
                0x30 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaVolumeUp           as u16, event_data: PressType::Press as i16})),
                0x32 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebHome            as u16, event_data: PressType::Press as i16})),
                0x35 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumForwardSlash         as u16, event_data: PressType::Press as i16})),
                0x38 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightAlt             as u16, event_data: PressType::Press as i16})),
                0x47 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyHome                 as u16, event_data: PressType::Press as i16})),
                0x48 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyUpArrow              as u16, event_data: PressType::Press as i16})),
                0x49 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPageUp               as u16, event_data: PressType::Press as i16})),
                0x4B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftArrow            as u16, event_data: PressType::Press as i16})),
                0x4D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightArrow           as u16, event_data: PressType::Press as i16})),
                0x4F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEnd                  as u16, event_data: PressType::Press as i16})),
                0x50 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDownArrow            as u16, event_data: PressType::Press as i16})),
                0x51 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPageDown             as u16, event_data: PressType::Press as i16})),
                0x52 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyInsert               as u16, event_data: PressType::Press as i16})),
                0x53 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDelete               as u16, event_data: PressType::Press as i16})),
                0x5B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftOperatingSystem  as u16, event_data: PressType::Press as i16})),
                0x5C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightOperatingSystem as u16, event_data: PressType::Press as i16})),
                0x5D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyApplication          as u16, event_data: PressType::Press as i16})),
                0x5E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPower                as u16, event_data: PressType::Press as i16})),
                0x5F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySleep                as u16, event_data: PressType::Press as i16})),
                0x63 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyWake                 as u16, event_data: PressType::Press as i16})),
                0x65 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebSearch          as u16, event_data: PressType::Press as i16})),
                0x66 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebFavorites       as u16, event_data: PressType::Press as i16})),
                0x67 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebRefresh         as u16, event_data: PressType::Press as i16})),
                0x68 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebStop            as u16, event_data: PressType::Press as i16})),
                0x69 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebForward         as u16, event_data: PressType::Press as i16})),
                0x6A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebBack            as u16, event_data: PressType::Press as i16})),
                0x6B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaMyComputer         as u16, event_data: PressType::Press as i16})),
                0x6C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaEmail              as u16, event_data: PressType::Press as i16})),
                0x6D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaSelect             as u16, event_data: PressType::Press as i16})),
                0x90 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaPreviousTrack      as u16, event_data: PressType::Unpress as i16})),
                0x99 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaNextTrack          as u16, event_data: PressType::Unpress as i16})),
                0x9C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumEnter                as u16, event_data: PressType::Unpress as i16})),
                0x9D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightControl         as u16, event_data: PressType::Unpress as i16})),
                0xA0 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaMute               as u16, event_data: PressType::Unpress as i16})),
                0xA1 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaCalculator         as u16, event_data: PressType::Unpress as i16})),
                0xA2 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaPlayPause          as u16, event_data: PressType::Unpress as i16})),
                0xA4 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaStop               as u16, event_data: PressType::Unpress as i16})),
                0xAE => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaVolumeDown         as u16, event_data: PressType::Unpress as i16})),
                0xB0 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaVolumeUp           as u16, event_data: PressType::Unpress as i16})),
                0xB2 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebHome            as u16, event_data: PressType::Unpress as i16})),
                0xB5 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumForwardSlash         as u16, event_data: PressType::Unpress as i16})),
                0xB8 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightAlt             as u16, event_data: PressType::Unpress as i16})),
                0xC7 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyHome                 as u16, event_data: PressType::Unpress as i16})),
                0xC8 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyUpArrow              as u16, event_data: PressType::Unpress as i16})),
                0xC9 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPageUp               as u16, event_data: PressType::Unpress as i16})),
                0xCB => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftArrow            as u16, event_data: PressType::Unpress as i16})),
                0xCD => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightArrow           as u16, event_data: PressType::Unpress as i16})),
                0xCF => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEnd                  as u16, event_data: PressType::Unpress as i16})),
                0xD0 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDownArrow            as u16, event_data: PressType::Unpress as i16})),
                0xD1 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPageDown             as u16, event_data: PressType::Unpress as i16})),
                0xD2 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyInsert               as u16, event_data: PressType::Unpress as i16})),
                0xD3 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDelete               as u16, event_data: PressType::Unpress as i16})),
                0xDB => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftOperatingSystem  as u16, event_data: PressType::Unpress as i16})),
                0xDC => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightOperatingSystem as u16, event_data: PressType::Unpress as i16})),
                0xDD => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyApplication          as u16, event_data: PressType::Unpress as i16})),
                0xDE => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPower                as u16, event_data: PressType::Unpress as i16})),
                0xDF => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySleep                as u16, event_data: PressType::Unpress as i16})),
                0xE3 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyWake                 as u16, event_data: PressType::Unpress as i16})),
                0xE5 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebSearch          as u16, event_data: PressType::Unpress as i16})),
                0xE6 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebFavorites       as u16, event_data: PressType::Unpress as i16})),
                0xE7 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebRefresh         as u16, event_data: PressType::Unpress as i16})),
                0xE8 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebStop            as u16, event_data: PressType::Unpress as i16})),
                0xE9 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebForward         as u16, event_data: PressType::Unpress as i16})),
                0xEA => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebBack            as u16, event_data: PressType::Unpress as i16})),
                0xEB => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaMyComputer         as u16, event_data: PressType::Unpress as i16})),
                0xEC => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaEmail              as u16, event_data: PressType::Unpress as i16})),
                0xED => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaSelect             as u16, event_data: PressType::Unpress as i16})),
                _ => Err("PS2 Scancode Set 1: Unrecognized scancode [Extension, Invalid].")
            }
            _ => Err("PS2 Scancode Set 1: Continuation error [Invalid, Unchecked].")
        },
        _ => Err("Unfinished")
    }
}

// Scancode Set 2 to input event translation
pub fn scancodes_2(scancodes: &[u8], device_id: u16) -> Result<Ps2Scan, &'static str> {
    match scancodes.len() {
        0x01 => match scancodes[0] {
            0x01 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF09          as u16, event_data: PressType::Press as i16})),
            0x03 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF05          as u16, event_data: PressType::Press as i16})),
            0x04 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF03          as u16, event_data: PressType::Press as i16})),
            0x05 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF01          as u16, event_data: PressType::Press as i16})),
            0x06 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF02          as u16, event_data: PressType::Press as i16})),
            0x07 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF12          as u16, event_data: PressType::Press as i16})),
            0x09 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF10          as u16, event_data: PressType::Press as i16})),
            0x0A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF08          as u16, event_data: PressType::Press as i16})),
            0x0B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF06          as u16, event_data: PressType::Press as i16})),
            0x0C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF04          as u16, event_data: PressType::Press as i16})),
            0x0D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyTab          as u16, event_data: PressType::Press as i16})),
            0x0E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyGrave        as u16, event_data: PressType::Press as i16})),
            0x11 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftAlt      as u16, event_data: PressType::Press as i16})),
            0x12 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftShift    as u16, event_data: PressType::Press as i16})),
            0x14 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftControl  as u16, event_data: PressType::Press as i16})),
            0x15 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyQ            as u16, event_data: PressType::Press as i16})),
            0x16 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key1            as u16, event_data: PressType::Press as i16})),
            0x1A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyZ            as u16, event_data: PressType::Press as i16})),
            0x1B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyS            as u16, event_data: PressType::Press as i16})),
            0x1C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyA            as u16, event_data: PressType::Press as i16})),
            0x1D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyW            as u16, event_data: PressType::Press as i16})),
            0x1E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key2            as u16, event_data: PressType::Press as i16})),
            0x21 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyC            as u16, event_data: PressType::Press as i16})),
            0x22 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyX            as u16, event_data: PressType::Press as i16})),
            0x23 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyD            as u16, event_data: PressType::Press as i16})),
            0x24 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyE            as u16, event_data: PressType::Press as i16})),
            0x25 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key4            as u16, event_data: PressType::Press as i16})),
            0x26 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key3            as u16, event_data: PressType::Press as i16})),
            0x29 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySpace        as u16, event_data: PressType::Press as i16})),
            0x2A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyV            as u16, event_data: PressType::Press as i16})),
            0x2B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF            as u16, event_data: PressType::Press as i16})),
            0x2C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyT            as u16, event_data: PressType::Press as i16})),
            0x2D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyR            as u16, event_data: PressType::Press as i16})),
            0x2E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key5            as u16, event_data: PressType::Press as i16})),
            0x31 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyN            as u16, event_data: PressType::Press as i16})),
            0x32 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyB            as u16, event_data: PressType::Press as i16})),
            0x33 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyH            as u16, event_data: PressType::Press as i16})),
            0x34 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyG            as u16, event_data: PressType::Press as i16})),
            0x35 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyY            as u16, event_data: PressType::Press as i16})),
            0x36 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key6            as u16, event_data: PressType::Press as i16})),
            0x3A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyM            as u16, event_data: PressType::Press as i16})),
            0x3B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyJ            as u16, event_data: PressType::Press as i16})),
            0x3C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyU            as u16, event_data: PressType::Press as i16})),
            0x3D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key7            as u16, event_data: PressType::Press as i16})),
            0x3E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key8            as u16, event_data: PressType::Press as i16})),
            0x41 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyComma        as u16, event_data: PressType::Press as i16})),
            0x42 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyK            as u16, event_data: PressType::Press as i16})),
            0x43 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyI            as u16, event_data: PressType::Press as i16})),
            0x44 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyO            as u16, event_data: PressType::Press as i16})),
            0x45 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key0            as u16, event_data: PressType::Press as i16})),
            0x46 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key9            as u16, event_data: PressType::Press as i16})),
            0x49 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPeriod       as u16, event_data: PressType::Press as i16})),
            0x4A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyForwardSlash as u16, event_data: PressType::Press as i16})),
            0x4B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyL            as u16, event_data: PressType::Press as i16})),
            0x4C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySemicolon    as u16, event_data: PressType::Press as i16})),
            0x4D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyP            as u16, event_data: PressType::Press as i16})),
            0x4E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDash         as u16, event_data: PressType::Press as i16})),
            0x52 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyQuote        as u16, event_data: PressType::Press as i16})),
            0x54 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyOpenBracket  as u16, event_data: PressType::Press as i16})),
            0x55 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEqual        as u16, event_data: PressType::Press as i16})),
            0x58 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyCapsLock     as u16, event_data: PressType::Press as i16})),
            0x59 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightShift   as u16, event_data: PressType::Press as i16})),
            0x5A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEnter        as u16, event_data: PressType::Press as i16})),
            0x5B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyCloseBracket as u16, event_data: PressType::Press as i16})),
            0x5D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyBackSlash    as u16, event_data: PressType::Press as i16})),
            0x66 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyBackspace    as u16, event_data: PressType::Press as i16})),
            0x69 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num1            as u16, event_data: PressType::Press as i16})),
            0x6B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num4            as u16, event_data: PressType::Press as i16})),
            0x6C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num7            as u16, event_data: PressType::Press as i16})),
            0x70 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num0            as u16, event_data: PressType::Press as i16})),
            0x71 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumPeriod       as u16, event_data: PressType::Press as i16})),
            0x72 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num2            as u16, event_data: PressType::Press as i16})),
            0x73 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num5            as u16, event_data: PressType::Press as i16})),
            0x74 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num6            as u16, event_data: PressType::Press as i16})),
            0x75 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num8            as u16, event_data: PressType::Press as i16})),
            0x76 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEscape       as u16, event_data: PressType::Press as i16})),
            0x77 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumLock         as u16, event_data: PressType::Press as i16})),
            0x78 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF11          as u16, event_data: PressType::Press as i16})),
            0x79 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumPlus         as u16, event_data: PressType::Press as i16})),
            0x7A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num3            as u16, event_data: PressType::Press as i16})),
            0x7B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumMinus        as u16, event_data: PressType::Press as i16})),
            0x7C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumMultiply     as u16, event_data: PressType::Press as i16})),
            0x7D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num9            as u16, event_data: PressType::Press as i16})),
            0x7E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyScrollLock   as u16, event_data: PressType::Press as i16})),
            0x83 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF07          as u16, event_data: PressType::Press as i16})),
            0xE0 => Ok(Ps2Scan::Continue),
            0xE1 => Ok(Ps2Scan::Continue),
            0xF0 => Ok(Ps2Scan::Continue),
            _ => Err("PS2 Scancode Set 2: Unrecognized scancode [Invalid].")
        }
        0x02 => match scancodes[0] {
            0xE0 => match scancodes[1] {
                0x10 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebSearch          as u16, event_data: PressType::Press as i16})),
                0x11 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightAlt             as u16, event_data: PressType::Press as i16})),
                0x14 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightControl         as u16, event_data: PressType::Press as i16})),
                0x15 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaPreviousTrack      as u16, event_data: PressType::Press as i16})),
                0x18 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebFavorites       as u16, event_data: PressType::Press as i16})),
                0x1F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftOperatingSystem  as u16, event_data: PressType::Press as i16})),
                0x20 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebRefresh         as u16, event_data: PressType::Press as i16})),
                0x21 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaVolumeDown         as u16, event_data: PressType::Press as i16})),
                0x23 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaMute               as u16, event_data: PressType::Press as i16})),
                0x27 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightOperatingSystem as u16, event_data: PressType::Press as i16})),
                0x28 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebStop            as u16, event_data: PressType::Press as i16})),
                0x29 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaCalculator         as u16, event_data: PressType::Press as i16})),
                0x2F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyMenu                 as u16, event_data: PressType::Press as i16})),
                0x30 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebForward         as u16, event_data: PressType::Press as i16})),
                0x32 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaVolumeUp           as u16, event_data: PressType::Press as i16})),
                0x34 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaPlayPause          as u16, event_data: PressType::Press as i16})),
                0x37 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPower                as u16, event_data: PressType::Press as i16})),
                0x38 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebBack            as u16, event_data: PressType::Press as i16})),
                0x3A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebHome            as u16, event_data: PressType::Press as i16})),
                0x3B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaStop               as u16, event_data: PressType::Press as i16})),
                0x3F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySleep                as u16, event_data: PressType::Press as i16})),
                0x40 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaMyComputer         as u16, event_data: PressType::Press as i16})),
                0x48 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaEmail              as u16, event_data: PressType::Press as i16})),
                0x4A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumForwardSlash         as u16, event_data: PressType::Press as i16})),
                0x4D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaNextTrack          as u16, event_data: PressType::Press as i16})),
                0x50 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaSelect             as u16, event_data: PressType::Press as i16})),
                0x5A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumEnter                as u16, event_data: PressType::Press as i16})),
                0x5E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyWake                 as u16, event_data: PressType::Press as i16})),
                0x69 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEnd                  as u16, event_data: PressType::Press as i16})),
                0x6B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftArrow            as u16, event_data: PressType::Press as i16})),
                0x6C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyHome                 as u16, event_data: PressType::Press as i16})),
                0x70 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyInsert               as u16, event_data: PressType::Press as i16})),
                0x71 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDelete               as u16, event_data: PressType::Press as i16})),
                0x72 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDownArrow            as u16, event_data: PressType::Press as i16})),
                0x74 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightArrow           as u16, event_data: PressType::Press as i16})),
                0x75 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyUpArrow              as u16, event_data: PressType::Press as i16})),
                0x7A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPageDown             as u16, event_data: PressType::Press as i16})),
                0x7D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPageUp               as u16, event_data: PressType::Press as i16})),
                0x12 => Ok(Ps2Scan::Continue),
                0xF0 => Ok(Ps2Scan::Continue),
                _ => Err("PS2 Scancode Set 2: Unrecognized scancode [Extension, Invalid].")
            }
            0xE1 => match scancodes[1] {
                0x14 => Ok(Ps2Scan::Continue),
                _ => Err("PS2 Scancode Set 2: Unrecognized scancode [Pause, Invalid].")
            }
            0xF0 => match scancodes[1] {
                0x01 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF09          as u16, event_data: PressType::Unpress as i16})),
                0x03 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF05          as u16, event_data: PressType::Unpress as i16})),
                0x04 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF03          as u16, event_data: PressType::Unpress as i16})),
                0x05 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF01          as u16, event_data: PressType::Unpress as i16})),
                0x06 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF02          as u16, event_data: PressType::Unpress as i16})),
                0x07 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF12          as u16, event_data: PressType::Unpress as i16})),
                0x09 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF10          as u16, event_data: PressType::Unpress as i16})),
                0x0A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF08          as u16, event_data: PressType::Unpress as i16})),
                0x0B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF06          as u16, event_data: PressType::Unpress as i16})),
                0x0C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF04          as u16, event_data: PressType::Unpress as i16})),
                0x0D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyTab          as u16, event_data: PressType::Unpress as i16})),
                0x0E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyGrave        as u16, event_data: PressType::Unpress as i16})),
                0x11 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftAlt      as u16, event_data: PressType::Unpress as i16})),
                0x12 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftShift    as u16, event_data: PressType::Unpress as i16})),
                0x14 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftControl  as u16, event_data: PressType::Unpress as i16})),
                0x15 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyQ            as u16, event_data: PressType::Unpress as i16})),
                0x16 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key1            as u16, event_data: PressType::Unpress as i16})),
                0x1A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyZ            as u16, event_data: PressType::Unpress as i16})),
                0x1B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyS            as u16, event_data: PressType::Unpress as i16})),
                0x1C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyA            as u16, event_data: PressType::Unpress as i16})),
                0x1D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyW            as u16, event_data: PressType::Unpress as i16})),
                0x1E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key2            as u16, event_data: PressType::Unpress as i16})),
                0x21 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyC            as u16, event_data: PressType::Unpress as i16})),
                0x22 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyX            as u16, event_data: PressType::Unpress as i16})),
                0x23 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyD            as u16, event_data: PressType::Unpress as i16})),
                0x24 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyE            as u16, event_data: PressType::Unpress as i16})),
                0x25 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key4            as u16, event_data: PressType::Unpress as i16})),
                0x26 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key3            as u16, event_data: PressType::Unpress as i16})),
                0x29 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySpace        as u16, event_data: PressType::Unpress as i16})),
                0x2A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyV            as u16, event_data: PressType::Unpress as i16})),
                0x2B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF            as u16, event_data: PressType::Unpress as i16})),
                0x2C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyT            as u16, event_data: PressType::Unpress as i16})),
                0x2D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyR            as u16, event_data: PressType::Unpress as i16})),
                0x2E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key5            as u16, event_data: PressType::Unpress as i16})),
                0x31 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyN            as u16, event_data: PressType::Unpress as i16})),
                0x32 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyB            as u16, event_data: PressType::Unpress as i16})),
                0x33 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyH            as u16, event_data: PressType::Unpress as i16})),
                0x34 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyG            as u16, event_data: PressType::Unpress as i16})),
                0x35 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyY            as u16, event_data: PressType::Unpress as i16})),
                0x36 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key6            as u16, event_data: PressType::Unpress as i16})),
                0x3A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyM            as u16, event_data: PressType::Unpress as i16})),
                0x3B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyJ            as u16, event_data: PressType::Unpress as i16})),
                0x3C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyU            as u16, event_data: PressType::Unpress as i16})),
                0x3D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key7            as u16, event_data: PressType::Unpress as i16})),
                0x3E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key8            as u16, event_data: PressType::Unpress as i16})),
                0x41 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyComma        as u16, event_data: PressType::Unpress as i16})),
                0x42 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyK            as u16, event_data: PressType::Unpress as i16})),
                0x43 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyI            as u16, event_data: PressType::Unpress as i16})),
                0x44 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyO            as u16, event_data: PressType::Unpress as i16})),
                0x45 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key0            as u16, event_data: PressType::Unpress as i16})),
                0x46 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Key9            as u16, event_data: PressType::Unpress as i16})),
                0x49 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPeriod       as u16, event_data: PressType::Unpress as i16})),
                0x4A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyForwardSlash as u16, event_data: PressType::Unpress as i16})),
                0x4B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyL            as u16, event_data: PressType::Unpress as i16})),
                0x4C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySemicolon    as u16, event_data: PressType::Unpress as i16})),
                0x4D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyP            as u16, event_data: PressType::Unpress as i16})),
                0x4E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDash         as u16, event_data: PressType::Unpress as i16})),
                0x52 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyQuote        as u16, event_data: PressType::Unpress as i16})),
                0x54 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyOpenBracket  as u16, event_data: PressType::Unpress as i16})),
                0x55 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEqual        as u16, event_data: PressType::Unpress as i16})),
                0x58 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyCapsLock     as u16, event_data: PressType::Unpress as i16})),
                0x59 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightShift   as u16, event_data: PressType::Unpress as i16})),
                0x5A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEnter        as u16, event_data: PressType::Unpress as i16})),
                0x5B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyCloseBracket as u16, event_data: PressType::Unpress as i16})),
                0x5D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyBackSlash    as u16, event_data: PressType::Unpress as i16})),
                0x66 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyBackspace    as u16, event_data: PressType::Unpress as i16})),
                0x69 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num1            as u16, event_data: PressType::Unpress as i16})),
                0x6B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num4            as u16, event_data: PressType::Unpress as i16})),
                0x6C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num7            as u16, event_data: PressType::Unpress as i16})),
                0x70 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num0            as u16, event_data: PressType::Unpress as i16})),
                0x71 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumPeriod       as u16, event_data: PressType::Unpress as i16})),
                0x72 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num2            as u16, event_data: PressType::Unpress as i16})),
                0x73 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num5            as u16, event_data: PressType::Unpress as i16})),
                0x74 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num6            as u16, event_data: PressType::Unpress as i16})),
                0x75 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num8            as u16, event_data: PressType::Unpress as i16})),
                0x76 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEscape       as u16, event_data: PressType::Unpress as i16})),
                0x77 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumLock         as u16, event_data: PressType::Unpress as i16})),
                0x78 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF11          as u16, event_data: PressType::Unpress as i16})),
                0x79 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumPlus         as u16, event_data: PressType::Unpress as i16})),
                0x7A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num3            as u16, event_data: PressType::Unpress as i16})),
                0x7B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumMinus        as u16, event_data: PressType::Unpress as i16})),
                0x7C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumMultiply     as u16, event_data: PressType::Unpress as i16})),
                0x7D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::Num9            as u16, event_data: PressType::Unpress as i16})),
                0x7E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyScrollLock   as u16, event_data: PressType::Unpress as i16})),
                0x83 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyF07          as u16, event_data: PressType::Unpress as i16})),
                _ => Err("PS2 Scancode Set 2: Unrecognized scancode [Unpress, Invalid].")
            }
            _ => Err("PS2 Scancode Set 2: Continuation error [Invalid, Unchecked].")
        }
        0x03 => match scancodes[0] {
            0xE0 => match scancodes[1] {
                0x12 => match scancodes[2] {
                    0xE0 => Ok(Ps2Scan::Continue),
                    _ => Err("PS/2 Scancode Set 2: Unrecognized scancode [Extended, Print Screen, Invalid].")
                }
                0xF0 => match scancodes[2] {
                    0x10 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebSearch          as u16, event_data: PressType::Unpress as i16})),
                    0x11 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightAlt             as u16, event_data: PressType::Unpress as i16})),
                    0x14 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightControl         as u16, event_data: PressType::Unpress as i16})),
                    0x15 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaPreviousTrack      as u16, event_data: PressType::Unpress as i16})),
                    0x18 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebFavorites       as u16, event_data: PressType::Unpress as i16})),
                    0x1F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftOperatingSystem  as u16, event_data: PressType::Unpress as i16})),
                    0x20 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebRefresh         as u16, event_data: PressType::Unpress as i16})),
                    0x21 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaVolumeDown         as u16, event_data: PressType::Unpress as i16})),
                    0x23 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaMute               as u16, event_data: PressType::Unpress as i16})),
                    0x27 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightOperatingSystem as u16, event_data: PressType::Unpress as i16})),
                    0x28 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebStop            as u16, event_data: PressType::Unpress as i16})),
                    0x29 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaCalculator         as u16, event_data: PressType::Unpress as i16})),
                    0x2F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyMenu                 as u16, event_data: PressType::Unpress as i16})),
                    0x30 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebForward         as u16, event_data: PressType::Unpress as i16})),
                    0x32 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaVolumeUp           as u16, event_data: PressType::Unpress as i16})),
                    0x34 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaPlayPause          as u16, event_data: PressType::Unpress as i16})),
                    0x37 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPower                as u16, event_data: PressType::Unpress as i16})),
                    0x38 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebBack            as u16, event_data: PressType::Unpress as i16})),
                    0x3A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaWebHome            as u16, event_data: PressType::Unpress as i16})),
                    0x3B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaStop               as u16, event_data: PressType::Unpress as i16})),
                    0x3F => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeySleep                as u16, event_data: PressType::Unpress as i16})),
                    0x40 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaMyComputer         as u16, event_data: PressType::Unpress as i16})),
                    0x48 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaEmail              as u16, event_data: PressType::Unpress as i16})),
                    0x4A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumForwardSlash         as u16, event_data: PressType::Unpress as i16})),
                    0x4D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaNextTrack          as u16, event_data: PressType::Unpress as i16})),
                    0x50 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::MediaSelect             as u16, event_data: PressType::Unpress as i16})),
                    0x5A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::NumEnter                as u16, event_data: PressType::Unpress as i16})),
                    0x5E => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyWake                 as u16, event_data: PressType::Unpress as i16})),
                    0x69 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyEnd                  as u16, event_data: PressType::Unpress as i16})),
                    0x6B => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyLeftArrow            as u16, event_data: PressType::Unpress as i16})),
                    0x6C => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyHome                 as u16, event_data: PressType::Unpress as i16})),
                    0x70 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyInsert               as u16, event_data: PressType::Unpress as i16})),
                    0x71 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDelete               as u16, event_data: PressType::Unpress as i16})),
                    0x72 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyDownArrow            as u16, event_data: PressType::Unpress as i16})),
                    0x74 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyRightArrow           as u16, event_data: PressType::Unpress as i16})),
                    0x75 => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyUpArrow              as u16, event_data: PressType::Unpress as i16})),
                    0x7A => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPageDown             as u16, event_data: PressType::Unpress as i16})),
                    0x7D => Ok(Ps2Scan::Finish(InputEvent{device_id, event_type: InputEventType::DigitalKey, event_id: KeyID::KeyPageUp               as u16, event_data: PressType::Unpress as i16})),
                    0x7C => Ok(Ps2Scan::Continue),
                    _ => Err("PS/2 Scancode Set 2: Unrecognized scancode [Extended, Unpress, Invalid].")
                }
                _ => Err("PS2 Scancode Set 2: Continuation error [Extended, Invalid, Unchecked].")
            }
            0xE1 => match scancodes[1] {
                0x14 => match scancodes[2] {
                    0x77 => Ok(Ps2Scan::Continue),
                    _ => Err("PS2 Scancode Set 2: Unrecognized scancode [Pause, Pause, Invalid].")
                }
                _ => Err("PS2 Scancode Set 2: Continuation error [Pause, Invalid, Unchecked].")
            }
            _ => Err("PS2 Scancode Set 2: Continuation error [Invalid, Unchecked, Unchecked].")
        }
        _ => Err("Unfinished")
    }
}
