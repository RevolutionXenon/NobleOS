// GLUON: PS/2
// Structs and objects related to the handling of the PS/2 controller and devices

use core::fmt::write;

use crate::*;

// PS/2 CONTROLLER
pub unsafe fn read_memory(address: u8) -> Result<u8, &'static str> {
    if address > 0x1F {return Err("PS/2 Controller: Memory read address out of bounds.")}
    PORT_PS2_COMMAND.write(address | 0x20);
    while PORT_PS2_STATUS.read() & 0x01 == 0 {}
    Ok(PORT_PS2_DATA.read())
}
pub unsafe fn write_memory(address: u8, data: u8) -> Result<(), &'static str> {
    if address > 0x1F {return Err("PS/2 Controller: Memory write address out of bounds.")}
    PORT_PS2_COMMAND.write(address | 0x60);
    while PORT_PS2_STATUS.read() & 0x02 != 0 {}
    PORT_PS2_DATA.write(data);
    Ok(())
}

pub unsafe fn test_controller() -> bool {PORT_PS2_COMMAND.write(0xAA); while PORT_PS2_STATUS.read() & 0x01 == 0 {} PORT_PS2_DATA.read() == 0x55}
pub unsafe fn test_port_1    () -> bool {PORT_PS2_COMMAND.write(0xAB); while PORT_PS2_STATUS.read() & 0x01 == 0 {} PORT_PS2_DATA.read() == 0x00}
pub unsafe fn test_port_2    () -> bool {PORT_PS2_COMMAND.write(0xA9); while PORT_PS2_STATUS.read() & 0x01 == 0 {} PORT_PS2_DATA.read() == 0x00}

pub unsafe fn enable_port1 () {PORT_PS2_COMMAND.write(0xAE)}
pub unsafe fn disable_port1() {PORT_PS2_COMMAND.write(0xAD)}
pub unsafe fn enable_port2 () {PORT_PS2_COMMAND.write(0xA8)}
pub unsafe fn disable_port2() {PORT_PS2_COMMAND.write(0xA7)}

pub unsafe fn enable_int_port1()  {write_memory(0x00, read_memory(0x00).unwrap() | 0x01).unwrap();}
pub unsafe fn disable_int_port1() {write_memory(0x00, read_memory(0x00).unwrap() & 0xFE).unwrap();}
pub unsafe fn enable_int_port2()  {write_memory(0x00, read_memory(0x00).unwrap() | 0x02).unwrap();}
pub unsafe fn disable_int_port2() {write_memory(0x00, read_memory(0x00).unwrap() & 0xFD).unwrap();}

pub unsafe fn poll_input() -> bool {PORT_PS2_STATUS.read() & 0x01 > 0}
pub unsafe fn read_input() -> u8   {PORT_PS2_DATA.read()}


//Input Event Enum
#[repr(u8)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum InputEvent {
    DigitalKey         (PressType, KeyID) = 0x01,
    DigitalButton      (PressType, u16)   = 0x02,
    AnalogPosition     (i16,       u16)   = 0x03,
    AnalogVelocity     (i16,       u16)   = 0x04,
    AnalogAcceleration (i16,       u16)   = 0x05,

}

//PressType Enum
#[repr(u16)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum PressType {
    Press   = 0x0001,
    Unpress = 0x0002,
}

//
#[repr(u16)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum KeyID {
    //Standard Keyboard Number Keys
    Key0 = 0x0000, Key1 = 0x0001, Key2 = 0x0002, Key3 = 0x0003, 
    Key4 = 0x0004, Key5 = 0x0005, Key6 = 0x0006, Key7 = 0x0007,
    Key8 = 0x0008, Key9 = 0x0009,
    //Standard Keyboard Alphabet Character Keys
    KeyQ = 0x0010, KeyW = 0x0011, KeyE = 0x0012, KeyR = 0x0013,
    KeyT = 0x0014, KeyY = 0x0015, KeyU = 0x0016, KeyI = 0x0017,
    KeyO = 0x0018, KeyP = 0x0019, KeyA = 0x001A, KeyS = 0x001B,
    KeyD = 0x001C, KeyF = 0x001D, KeyG = 0x001E, KeyH = 0x001F,
    KeyJ = 0x0020, KeyK = 0x0021, KeyL = 0x0022, KeyZ = 0x0023,
    KeyX = 0x0024, KeyC = 0x0025, KeyV = 0x0026, KeyB = 0x0027,
    KeyN = 0x0028, KeyM = 0x0029,
    //Standard Keyboard Punctuation Character Keys
    KeyGrave        = 0x0030, KeyMinus        = 0x0031, KeyEqual        = 0x0032, KeyBackspace    = 0x0033, 
    KeyTab          = 0x0034, KeyOpenBracket  = 0x0035, KeyCloseBracket = 0x0036, KeyBackSlash    = 0x0037, 
    KeySemicolon    = 0x0038, KeyQuote        = 0x0039, KeyEnter        = 0x003A, KeyComma        = 0x003B,
    KeyPeriod       = 0x003C, KeyForwardSlash = 0x003D,
    //Standard Keyboard Modifier Keys
    KeyScrollLock   = 0x0040, KeyCapsLock     = 0x0041, KeyLeftShift    = 0x0042, KeyRightShift   = 0x0043,
    KeyLeftControl  = 0x0044, KeyRightControl = 0x0045, KeyLeftAlt      = 0x0046, KeyRightAlt     = 0x0047,
    KeyLeftOperatingSystem                    = 0x0048, KeyRightOperatingSystem                   = 0x0049,
    //Standard Keyboard Action Keys
    KeyEscape       = 0x0050, KeyPrintScreen  = 0x0051, KeyPause        = 0x0052, KeyHome         = 0x0053,
    KeyEnd          = 0x0054, KeyDelete       = 0x0055, KeyPageUp       = 0x0056, KeyPageDown     = 0x0057, 
    KeyInsert       = 0x0058, KeyUpArrow      = 0x0059, KeyDownArrow    = 0x005A, KeyLeftArrow    = 0x005B,
    KeyRightArrow   = 0x005C,
    //Standard Keyboard Function Keys
    KeyF01 = 0x0060, KeyF02 = 0x0061, KeyF03 = 0x0062, KeyF04 = 0x0063,
    KeyF05 = 0x0064, KeyF06 = 0x0065, KeyF07 = 0x0066, KeyF08 = 0x0067,
    KeyF09 = 0x0068, KeyF10 = 0x0069, KeyF11 = 0x006A, KeyF12 = 0x006B,
    KeyF13 = 0x006C, KeyF14 = 0x006D, KeyF15 = 0x006E, KeyF16 = 0x007F,
    KeyF17 = 0x0070, KeyF18 = 0x0071, KeyF19 = 0x0072, KeyF20 = 0x0073,
    KeyF21 = 0x0074, KeyF22 = 0x0075, KeyF23 = 0x0076, KeyF24 = 0x0077,

    //Standard Numpad Number Keys
    Num0 = 0x0080, Num1 = 0x0081, Num2 = 0x0082, Num3 = 0x0083,
    Num4 = 0x0084, Num5 = 0x0085, Num6 = 0x0086, Num7 = 0x0087,
    Num8 = 0x0088, Num9 = 0x0089,
    //Standard Numpad Modifier Keys
    NumLock         = 0x0090,
    //Standard Numpad Punctuation Keys
    NumForwardSlash = 0x00A0, NumMultiply     = 0x00A1, NumMinus        = 0x00A2, NumPlus         = 0x00A3,
    NumEnter        = 0x00A4,

    //Non standard Keys
    KeyNonUSPound = 0xFF00,
    KeyDeleteForward,
    //
    Num00,
    Num000,
    NumPeriod,
    NumEqual,
    NumComma,
    NumEqualSign,
    NumOpenParenthesis,
    NumCloseParenthesis,
    NumOpenBrace,
    NumCloseBrace,
    NumTab,
    NumBackspace,
    NumA,
    NumB,
    NumC,
    NumD,
    NumE,
    NumF,
    NumExclusiveOr,
    NumExponent,
    NumPercent,
    NumLessThan,
    NumGreaterThan,
    NumLogicalAnd,
    NumBooleanAnd,
    NumLogicalOr,
    NumBooleanOr,
    NumColon,
    NumPound,
    NumSpace,
    NumAddress,
    NumNot,
    NumMemoryStore,
    NumMemoryRecall,
    NumMemoryClear,
    NumMemoryAdd,
    NumMemorySubtract,
    NumMemoryMultiply,
    NumMemoryDivide,
    NumPlusAndMinus,
    NumClear,
    NumClearEntry,
    NumBinary,
    NumOctal,
    NumDecimal,
    NumHexadecimal,
    //
    KeyNonUSBackSlash,
    KeyApplication,
    KeyPower,
    KeyExecute,
    KeyHelp,
    KeyMenu,
    KeySelect,
    KeyStop,
    KeyAgain,
    KeyUndo,
    KeyCut,
    KeyCopy,
    KeyPaste,
    KeyFind,
    //
    MediaMute,
    MediaVolumeUp,
    MediaVolumeDown,
    //
    KeyCapsLockHold,
    KeyNumLockHold,
    KeyScrollLockHold,
    //
    KeyInternational1,
    KeyInternational2,
    KeyInternational3,
    KeyInternational4,
    KeyInternational5,
    KeyInternational6,
    KeyInternational7,
    KeyInternational8,
    KeyInternational9,
    //
    KeyLanguage1,
    KeyLanguage2,
    KeyLanguage3,
    KeyLanguage4,
    KeyLanguage5,
    KeyLanguage6,
    KeyLanguage7,
    KeyLanguage8,
    KeyLanguage9,
    //
    KeyAlternateErase,
    KeyAttention,
    KeyCancel,
    KeyClear,
    KeyPrior,
    KeyReturn,
    KeySeparator,
    KeyOut,
    KeyOperatingSystem,
    KeyClearAgain,
    KeyControlSelect,
    KeyExecuteSelect,
    //
    KeyThousandsSeparator,
    KeyDecimalSeparator,
    KeyCurrencyUnit,
    KeyCurrencySubunit,
}
