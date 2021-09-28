// GLUON: PS/2
// Structs and objects related to the handling of the PS/2 controller and devices

use core::{fmt::write, num};

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

//Physical Key ID Enum
#[repr(u16)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum KeyID {
    //Standard Keyboard Number Keys
    Key0               = 0x0000, Key1               = 0x0001, Key2               = 0x0002, Key3               = 0x0003, 
    Key4               = 0x0004, Key5               = 0x0005, Key6               = 0x0006, Key7               = 0x0007,
    Key8               = 0x0008, Key9               = 0x0009,
    //Standard Keyboard Alphabet Character Keys
    KeyQ               = 0x0010, KeyW               = 0x0011, KeyE               = 0x0012, KeyR               = 0x0013,
    KeyT               = 0x0014, KeyY               = 0x0015, KeyU               = 0x0016, KeyI               = 0x0017,
    KeyO               = 0x0018, KeyP               = 0x0019, KeyA               = 0x001A, KeyS               = 0x001B,
    KeyD               = 0x001C, KeyF               = 0x001D, KeyG               = 0x001E, KeyH               = 0x001F,
    KeyJ               = 0x0020, KeyK               = 0x0021, KeyL               = 0x0022, KeyZ               = 0x0023,
    KeyX               = 0x0024, KeyC               = 0x0025, KeyV               = 0x0026, KeyB               = 0x0027,
    KeyN               = 0x0028, KeyM               = 0x0029,
    //Standard Keyboard Punctuation Character Keys
    KeyGrave           = 0x0030, KeyDash            = 0x0031, KeyEqual           = 0x0032, KeyBackspace       = 0x0033, 
    KeyTab             = 0x0034, KeyOpenBracket     = 0x0035, KeyCloseBracket    = 0x0036, KeyBackSlash       = 0x0037, 
    KeySemicolon       = 0x0038, KeyQuote           = 0x0039, KeyEnter           = 0x003A, KeyComma           = 0x003B,
    KeyPeriod          = 0x003C, KeyForwardSlash    = 0x003D, KeySpace           = 0x003E,
    //Standard Keyboard Modifier Keys
    KeyScrollLock      = 0x0040, KeyCapsLock        = 0x0041, KeyLeftShift       = 0x0042, KeyRightShift      = 0x0043,
    KeyLeftControl     = 0x0044, KeyRightControl    = 0x0045, KeyLeftAlt         = 0x0046, KeyRightAlt        = 0x0047,
    KeyLeftOperatingSystem                          = 0x0048, KeyRightOperatingSystem                         = 0x0049,
    //Standard Keyboard Action Keys
    KeyEscape          = 0x0050, KeyPrintScreen     = 0x0051, KeyPause           = 0x0052, KeyHome            = 0x0053,
    KeyEnd             = 0x0054, KeyDelete          = 0x0055, KeyPageUp          = 0x0056, KeyPageDown        = 0x0057, 
    KeyInsert          = 0x0058, KeyUpArrow         = 0x0059, KeyDownArrow       = 0x005A, KeyLeftArrow       = 0x005B,
    KeyRightArrow      = 0x005C,
    //Standard Keyboard Function Keys
    KeyF01             = 0x0060, KeyF02             = 0x0061, KeyF03             = 0x0062, KeyF04             = 0x0063,
    KeyF05             = 0x0064, KeyF06             = 0x0065, KeyF07             = 0x0066, KeyF08             = 0x0067,
    KeyF09             = 0x0068, KeyF10             = 0x0069, KeyF11             = 0x006A, KeyF12             = 0x006B,
    KeyF13             = 0x006C, KeyF14             = 0x006D, KeyF15             = 0x006E, KeyF16             = 0x007F,
    KeyF17             = 0x0070, KeyF18             = 0x0071, KeyF19             = 0x0072, KeyF20             = 0x0073,
    KeyF21             = 0x0074, KeyF22             = 0x0075, KeyF23             = 0x0076, KeyF24             = 0x0077,

    //Standard Numpad Number Keys
    Num0               = 0x0080, Num1               = 0x0081, Num2               = 0x0082, Num3               = 0x0083,
    Num4               = 0x0084, Num5               = 0x0085, Num6               = 0x0086, Num7               = 0x0087,
    Num8               = 0x0088, Num9               = 0x0089,
    //Standard Numpad Modifier Keys
    NumLock            = 0x0090,
    //Standard Numpad Punctuation Keys
    NumForwardSlash    = 0x00A0, NumMultiply        = 0x00A1, NumMinus           = 0x00A2, NumPlus            = 0x00A3,
    NumEnter           = 0x00A4,

    //Standard Multimedia Keys
    MediaPlayPause     = 0x00B0, MediaNextTrack     = 0x00B1, MediaPreviousTrack = 0x00B2, MediaStop          = 0x00B3, 
    MediaVolumeDown    = 0x00B4, MediaVolumeUp      = 0x00B5, MediaMute          = 0x00B6, MediaMyComputer    = 0x00B7, 
    MediaEmail         = 0x00B8, MediaSelect        = 0x00B9, MediaWebStop       = 0x00BA, MediaWebForward    = 0x00BB,
    MediaWebSearch     = 0x00BC, MediaWebBack       = 0x00BD, MediaWebHome       = 0x00BE, MediaWebFavorites  = 0x00BF, 
    MediaWebRefresh    = 0x00C0, MediaCalculator    = 0x00C1,

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
    KeySleep,
    KeyWake,
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

//Key or Char
pub enum KeyStr {
    Key(KeyID),
    Str(&'static str),
}

// US Qwerty Translation
pub fn us_qwerty(key: KeyID, alphabet_modify: bool, numpad_modify: bool) -> KeyStr {
    match (key, alphabet_modify, numpad_modify) {
        //Standard Keyboard Number Keys
        (KeyID::Key1,            false, _) => KeyStr::Str("1"),  (KeyID::Key1,            true, _) => KeyStr::Str("!"),
        (KeyID::Key2,            false, _) => KeyStr::Str("2"),  (KeyID::Key2,            true, _) => KeyStr::Str("@"),
        (KeyID::Key3,            false, _) => KeyStr::Str("3"),  (KeyID::Key3,            true, _) => KeyStr::Str("#"),
        (KeyID::Key4,            false, _) => KeyStr::Str("4"),  (KeyID::Key4,            true, _) => KeyStr::Str("$"),
        (KeyID::Key5,            false, _) => KeyStr::Str("5"),  (KeyID::Key5,            true, _) => KeyStr::Str("%"),
        (KeyID::Key6,            false, _) => KeyStr::Str("6"),  (KeyID::Key6,            true, _) => KeyStr::Str("^"),
        (KeyID::Key7,            false, _) => KeyStr::Str("7"),  (KeyID::Key7,            true, _) => KeyStr::Str("&"),
        (KeyID::Key8,            false, _) => KeyStr::Str("8"),  (KeyID::Key8,            true, _) => KeyStr::Str("*"),
        (KeyID::Key9,            false, _) => KeyStr::Str("9"),  (KeyID::Key9,            true, _) => KeyStr::Str("("),
        (KeyID::Key0,            false, _) => KeyStr::Str("0"),  (KeyID::Key0,            true, _) => KeyStr::Str(")"),
        //Standard Keyboard Alphabet Character Keys
        (KeyID::KeyQ,            false, _) => KeyStr::Str("q"),  (KeyID::KeyQ,            true, _) => KeyStr::Str("Q"),
        (KeyID::KeyW,            false, _) => KeyStr::Str("w"),  (KeyID::KeyW,            true, _) => KeyStr::Str("W"),
        (KeyID::KeyE,            false, _) => KeyStr::Str("e"),  (KeyID::KeyE,            true, _) => KeyStr::Str("E"),
        (KeyID::KeyR,            false, _) => KeyStr::Str("r"),  (KeyID::KeyR,            true, _) => KeyStr::Str("R"),
        (KeyID::KeyT,            false, _) => KeyStr::Str("t"),  (KeyID::KeyT,            true, _) => KeyStr::Str("T"),
        (KeyID::KeyY,            false, _) => KeyStr::Str("y"),  (KeyID::KeyY,            true, _) => KeyStr::Str("Y"),
        (KeyID::KeyU,            false, _) => KeyStr::Str("u"),  (KeyID::KeyU,            true, _) => KeyStr::Str("U"),
        (KeyID::KeyI,            false, _) => KeyStr::Str("i"),  (KeyID::KeyI,            true, _) => KeyStr::Str("I"),
        (KeyID::KeyO,            false, _) => KeyStr::Str("o"),  (KeyID::KeyO,            true, _) => KeyStr::Str("O"),
        (KeyID::KeyP,            false, _) => KeyStr::Str("p"),  (KeyID::KeyP,            true, _) => KeyStr::Str("P"),
        (KeyID::KeyA,            false, _) => KeyStr::Str("a"),  (KeyID::KeyA,            true, _) => KeyStr::Str("A"),
        (KeyID::KeyS,            false, _) => KeyStr::Str("s"),  (KeyID::KeyS,            true, _) => KeyStr::Str("S"),
        (KeyID::KeyD,            false, _) => KeyStr::Str("d"),  (KeyID::KeyD,            true, _) => KeyStr::Str("D"),
        (KeyID::KeyF,            false, _) => KeyStr::Str("f"),  (KeyID::KeyF,            true, _) => KeyStr::Str("F"),
        (KeyID::KeyG,            false, _) => KeyStr::Str("g"),  (KeyID::KeyG,            true, _) => KeyStr::Str("G"),
        (KeyID::KeyH,            false, _) => KeyStr::Str("h"),  (KeyID::KeyH,            true, _) => KeyStr::Str("H"),
        (KeyID::KeyJ,            false, _) => KeyStr::Str("j"),  (KeyID::KeyJ,            true, _) => KeyStr::Str("J"),
        (KeyID::KeyK,            false, _) => KeyStr::Str("k"),  (KeyID::KeyK,            true, _) => KeyStr::Str("K"),
        (KeyID::KeyL,            false, _) => KeyStr::Str("l"),  (KeyID::KeyL,            true, _) => KeyStr::Str("L"),
        (KeyID::KeyZ,            false, _) => KeyStr::Str("z"),  (KeyID::KeyZ,            true, _) => KeyStr::Str("Z"),
        (KeyID::KeyX,            false, _) => KeyStr::Str("x"),  (KeyID::KeyX,            true, _) => KeyStr::Str("X"),
        (KeyID::KeyC,            false, _) => KeyStr::Str("c"),  (KeyID::KeyC,            true, _) => KeyStr::Str("C"),
        (KeyID::KeyV,            false, _) => KeyStr::Str("v"),  (KeyID::KeyV,            true, _) => KeyStr::Str("V"),
        (KeyID::KeyB,            false, _) => KeyStr::Str("b"),  (KeyID::KeyB,            true, _) => KeyStr::Str("B"),
        (KeyID::KeyN,            false, _) => KeyStr::Str("n"),  (KeyID::KeyN,            true, _) => KeyStr::Str("N"),
        (KeyID::KeyM,            false, _) => KeyStr::Str("m"),  (KeyID::KeyM,            true, _) => KeyStr::Str("M"),
        //Standard Keyboard Punctuation Keys
        (KeyID::KeyGrave,        false, _) => KeyStr::Str("`"),  (KeyID::KeyGrave,        true, _) => KeyStr::Str("~"),
        (KeyID::KeyDash,         false, _) => KeyStr::Str("-"),  (KeyID::KeyDash,         true, _) => KeyStr::Str("_"),
        (KeyID::KeyEqual,        false, _) => KeyStr::Str("="),  (KeyID::KeyEqual,        true, _) => KeyStr::Str("+"),
        (KeyID::KeyBackSlash,    false, _) => KeyStr::Str("\\"), (KeyID::KeyBackSlash,    true, _) => KeyStr::Str("|"),
        (KeyID::KeyOpenBracket,  false, _) => KeyStr::Str("["),  (KeyID::KeyOpenBracket,  true, _) => KeyStr::Str("{"),
        (KeyID::KeyCloseBracket, false, _) => KeyStr::Str("]"),  (KeyID::KeyCloseBracket, true, _) => KeyStr::Str("}"),
        (KeyID::KeySemicolon,    false, _) => KeyStr::Str(";"),  (KeyID::KeySemicolon,    true, _) => KeyStr::Str(":"),
        (KeyID::KeyComma,        false, _) => KeyStr::Str(","),  (KeyID::KeyComma,        true, _) => KeyStr::Str("<"),
        (KeyID::KeyPeriod,       false, _) => KeyStr::Str("."),  (KeyID::KeyPeriod,       true, _) => KeyStr::Str(">"),
        (KeyID::KeyForwardSlash, false, _) => KeyStr::Str("/"),  (KeyID::KeyForwardSlash, true, _) => KeyStr::Str("?"),
        (KeyID::KeyBackspace, _, _) => KeyStr::Str("\x08"),
        (KeyID::KeyTab,       _, _) => KeyStr::Str("\t"),
        (KeyID::KeySpace,     _, _) => KeyStr::Str(" "),
        (KeyID::KeyEnter,     _, _) => KeyStr::Str("\n"),
        //Numpad
        (KeyID::NumForwardSlash, _, _) => KeyStr::Str("/"),
        (KeyID::NumMultiply,     _, _) => KeyStr::Str("*"),
        (KeyID::NumMinus,        _, _) => KeyStr::Str("-"),
        (KeyID::NumPlus,         _, _) => KeyStr::Str("+"),
        (KeyID::NumEnter,        _, _) => KeyStr::Str("\n"),
        (KeyID::NumPeriod, _, false) => KeyStr::Str("."), (KeyID::NumPeriod, _, true) => KeyStr::Key(KeyID::KeyDelete),
        (KeyID::Num0,      _, false) => KeyStr::Str("0"), (KeyID::Num0,      _, true) => KeyStr::Key(KeyID::KeyInsert),
        (KeyID::Num1,      _, false) => KeyStr::Str("1"), (KeyID::Num1,      _, true) => KeyStr::Key(KeyID::KeyEnd),
        (KeyID::Num2,      _, false) => KeyStr::Str("2"), (KeyID::Num2,      _, true) => KeyStr::Key(KeyID::KeyDownArrow),
        (KeyID::Num3,      _, false) => KeyStr::Str("3"), (KeyID::Num3,      _, true) => KeyStr::Key(KeyID::KeyPageDown),
        (KeyID::Num4,      _, false) => KeyStr::Str("4"), (KeyID::Num4,      _, true) => KeyStr::Key(KeyID::KeyLeftArrow),
        (KeyID::Num5,      _, false) => KeyStr::Str("5"),
        (KeyID::Num6,      _, false) => KeyStr::Str("6"), (KeyID::Num6,      _, true) => KeyStr::Key(KeyID::KeyRightArrow),
        (KeyID::Num7,      _, false) => KeyStr::Str("7"), (KeyID::Num7,      _, true) => KeyStr::Key(KeyID::KeyHome),
        (KeyID::Num8,      _, false) => KeyStr::Str("8"), (KeyID::Num8,      _, true) => KeyStr::Key(KeyID::KeyUpArrow),
        (KeyID::Num9,      _, false) => KeyStr::Str("9"), (KeyID::Num9,      _, true) => KeyStr::Key(KeyID::KeyPageUp),
        //Non Symbolic Key
        _ => KeyStr::Key(key)
    }
}

// PS/2 SCAN CODE SET 2 TO KEY CONVERSION FUNCTION
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum Ps2Scan {
    Finish(InputEvent),
    Continue,
}

pub fn scancodes_2(scancodes: &[u8]) -> Result<Ps2Scan, &'static str> {
    match scancodes.len() {
        0x1 => match scancodes[0] {
            0x01 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF09))),
            0x03 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF05))),
            0x04 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF03))),
            0x05 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF01))),
            0x06 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF02))),
            0x07 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF12))),
            0x09 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF10))),
            0x0A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF08))),
            0x0B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF06))),
            0x0C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF04))),
            0x0D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyTab))),
            0x0E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyGrave))),
            0x11 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyLeftAlt))),
            0x12 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyLeftShift))),
            0x14 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyLeftControl))),
            0x15 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyQ))),
            0x16 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key1))),
            0x1A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyZ))),
            0x1B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyS))),
            0x1C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyA))),
            0x1D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyW))),
            0x1E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key2))),
            0x21 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyC))),
            0x22 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyX))),
            0x23 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyD))),
            0x24 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyE))),
            0x25 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key4))),
            0x26 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key3))),
            0x29 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeySpace))),
            0x2A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyV))),
            0x2B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF))),
            0x2C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyT))),
            0x2D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyR))),
            0x2E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key5))),
            0x31 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyN))),
            0x32 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyB))),
            0x33 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyH))),
            0x34 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyG))),
            0x35 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyY))),
            0x36 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key6))),
            0x3A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyM))),
            0x3B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyJ))),
            0x3C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyU))),
            0x3D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key7))),
            0x3E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key8))),
            0x41 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyComma))),
            0x42 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyK))),
            0x43 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyI))),
            0x44 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyO))),
            0x45 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key0))),
            0x46 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Key9))),
            0x49 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyPeriod))),
            0x4A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyForwardSlash))),
            0x4B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyL))),
            0x4C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeySemicolon))),
            0x4D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyP))),
            0x4E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyDash))),
            0x52 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyQuote))),
            0x54 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyOpenBracket))),
            0x55 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyEqual))),
            0x58 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyCapsLock))),
            0x59 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyRightShift))),
            0x5A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyEnter))),
            0x5B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyCloseBracket))),
            0x5D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyBackSlash))),
            0x66 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyBackspace))),
            0x69 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num1))),
            0x6B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num4))),
            0x6C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num7))),
            0x70 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num0))),
            0x71 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::NumPeriod))),
            0x72 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num2))),
            0x73 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num5))),
            0x74 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num6))),
            0x75 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num8))),
            0x76 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyEscape))),
            0x77 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::NumLock))),
            0x78 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF11))),
            0x79 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::NumPlus))),
            0x7A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num3))),
            0x7B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::NumMinus))),
            0x7C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::NumMultiply))),
            0x7D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::Num9))),
            0x7E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyScrollLock))),
            0x83 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyF07))),
            0xE0 => Ok(Ps2Scan::Continue),
            0xE1 => Ok(Ps2Scan::Continue),
            0xF0 => Ok(Ps2Scan::Continue),
            _ => Err("PS2 Scancode Set 2: Unrecognized scancode [Invalid].")
        }
        0x02 => match scancodes[0] {
            0xE0 => match scancodes[1] {
                0x10 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaWebSearch))),
                0x11 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyRightAlt))),
                0x14 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyRightControl))),
                0x15 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaPreviousTrack))),
                0x18 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaWebFavorites))),
                0x1F => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyLeftOperatingSystem))),
                0x20 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaWebRefresh))),
                0x21 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaVolumeDown))),
                0x23 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaMute))),
                0x27 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyRightOperatingSystem))),
                0x28 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaWebStop))),
                0x29 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaCalculator))),
                0x2F => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyMenu))),
                0x30 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaWebForward))),
                0x32 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaVolumeUp))),
                0x34 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaPlayPause))),
                0x37 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyPower))),
                0x38 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaWebBack))),
                0x3A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaWebHome))),
                0x3B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaStop))),
                0x3F => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeySleep))),
                0x40 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaMyComputer))),
                0x48 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaEmail))),
                0x4A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::NumForwardSlash))),
                0x4D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaNextTrack))),
                0x50 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::MediaSelect))),
                0x5A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::NumEnter))),
                0x5E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyWake))),
                0x69 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyEnd))),
                0x6B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyLeftArrow))),
                0x6C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyHome))),
                0x70 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyInsert))),
                0x71 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyDelete))),
                0x72 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyDownArrow))),
                0x74 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyRightArrow))),
                0x75 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyUpArrow))),
                0x7A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyPageDown))),
                0x7D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Press, KeyID::KeyPageUp))),
                0x12 => Ok(Ps2Scan::Continue),
                0xF0 => Ok(Ps2Scan::Continue),
                _ => Err("PS2 Scancode Set 2: Unrecognized scancode [Extension, Invalid].")
            }
            0xE1 => match scancodes[1] {
                0x14 => Ok(Ps2Scan::Continue),
                _ => Err("PS2 Scancode Set 2: Unrecognized scancode [Pause, Invalid].")
            }
            0xF0 => match scancodes[1] {
                0x01 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF09))),
                0x03 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF05))),
                0x04 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF03))),
                0x05 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF01))),
                0x06 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF02))),
                0x07 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF12))),
                0x09 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF10))),
                0x0A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF08))),
                0x0B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF06))),
                0x0C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF04))),
                0x0D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyTab))),
                0x0E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyGrave))),
                0x11 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyLeftAlt))),
                0x12 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyLeftShift))),
                0x14 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyLeftControl))),
                0x15 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyQ))),
                0x16 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key1))),
                0x1A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyZ))),
                0x1B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyS))),
                0x1C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyA))),
                0x1D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyW))),
                0x1E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key2))),
                0x21 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyC))),
                0x22 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyX))),
                0x23 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyD))),
                0x24 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyE))),
                0x25 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key4))),
                0x26 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key3))),
                0x29 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeySpace))),
                0x2A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyV))),
                0x2B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF))),
                0x2C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyT))),
                0x2D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyR))),
                0x2E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key5))),
                0x31 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyN))),
                0x32 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyB))),
                0x33 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyH))),
                0x34 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyG))),
                0x35 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyY))),
                0x36 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key6))),
                0x3A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyM))),
                0x3B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyJ))),
                0x3C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyU))),
                0x3D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key7))),
                0x3E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key8))),
                0x41 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyComma))),
                0x42 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyK))),
                0x43 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyI))),
                0x44 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyO))),
                0x45 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key0))),
                0x46 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Key9))),
                0x49 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyPeriod))),
                0x4A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyForwardSlash))),
                0x4B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyL))),
                0x4C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeySemicolon))),
                0x4D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyP))),
                0x4E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyDash))),
                0x52 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyQuote))),
                0x54 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyOpenBracket))),
                0x55 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyEqual))),
                0x58 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyCapsLock))),
                0x59 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyRightShift))),
                0x5A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyEnter))),
                0x5B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyCloseBracket))),
                0x5D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyBackSlash))),
                0x66 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyBackspace))),
                0x69 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num1))),
                0x6B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num4))),
                0x6C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num7))),
                0x70 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num0))),
                0x71 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::NumPeriod))),
                0x72 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num2))),
                0x73 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num5))),
                0x74 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num6))),
                0x75 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num8))),
                0x76 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyEscape))),
                0x77 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::NumLock))),
                0x78 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF11))),
                0x79 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::NumPlus))),
                0x7A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num3))),
                0x7B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::NumMinus))),
                0x7C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::NumMultiply))),
                0x7D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::Num9))),
                0x7E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyScrollLock))),
                0x83 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyF07))),
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
                    0x10 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaWebSearch))),
                    0x11 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyRightAlt))),
                    0x14 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyRightControl))),
                    0x15 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaPreviousTrack))),
                    0x18 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaWebFavorites))),
                    0x1F => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyLeftOperatingSystem))),
                    0x20 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaWebRefresh))),
                    0x21 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaVolumeDown))),
                    0x23 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaMute))),
                    0x27 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyRightOperatingSystem))),
                    0x28 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaWebStop))),
                    0x29 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaCalculator))),
                    0x2F => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyMenu))),
                    0x30 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaWebForward))),
                    0x32 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaVolumeUp))),
                    0x34 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaPlayPause))),
                    0x37 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyPower))),
                    0x38 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaWebBack))),
                    0x3A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaWebHome))),
                    0x3B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaStop))),
                    0x3F => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeySleep))),
                    0x40 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaMyComputer))),
                    0x48 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaEmail))),
                    0x4A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::NumForwardSlash))),
                    0x4D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaNextTrack))),
                    0x50 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::MediaSelect))),
                    0x5A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::NumEnter))),
                    0x5E => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyWake))),
                    0x69 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyEnd))),
                    0x6B => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyLeftArrow))),
                    0x6C => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyHome))),
                    0x70 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyInsert))),
                    0x71 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyDelete))),
                    0x72 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyDownArrow))),
                    0x74 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyRightArrow))),
                    0x75 => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyUpArrow))),
                    0x7A => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyPageDown))),
                    0x7D => Ok(Ps2Scan::Finish(InputEvent::DigitalKey(PressType::Unpress, KeyID::KeyPageUp))),
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
