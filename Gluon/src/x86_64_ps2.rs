// GLUON: x86-64 PS/2
// Structs and objects related to the handling of the PS/2 controller and devices


// HEADER
//Imports
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
pub unsafe fn test_port_1()     -> bool {PORT_PS2_COMMAND.write(0xAB); while PORT_PS2_STATUS.read() & 0x01 == 0 {} PORT_PS2_DATA.read() == 0x00}
pub unsafe fn test_port_2()     -> bool {PORT_PS2_COMMAND.write(0xA9); while PORT_PS2_STATUS.read() & 0x01 == 0 {} PORT_PS2_DATA.read() == 0x00}

pub unsafe fn enable_port1()  {PORT_PS2_COMMAND.write(0xAE)}
pub unsafe fn disable_port1() {PORT_PS2_COMMAND.write(0xAD)}
pub unsafe fn enable_port2()  {PORT_PS2_COMMAND.write(0xA8)}
pub unsafe fn disable_port2() {PORT_PS2_COMMAND.write(0xA7)}

pub unsafe fn enable_int_port1()  {write_memory(0x00, read_memory(0x00).unwrap() | 0x01).unwrap();}
pub unsafe fn disable_int_port1() {write_memory(0x00, read_memory(0x00).unwrap() & 0xFE).unwrap();}
pub unsafe fn enable_int_port2()  {write_memory(0x00, read_memory(0x00).unwrap() | 0x02).unwrap();}
pub unsafe fn disable_int_port2() {write_memory(0x00, read_memory(0x00).unwrap() & 0xFD).unwrap();}

pub unsafe fn poll_input() -> bool {PORT_PS2_STATUS.read() & 0x01 > 0}
pub unsafe fn read_input() -> u8   {PORT_PS2_DATA.read()}

pub unsafe fn read_status() -> u8   {PORT_PS2_STATUS.read()}


// INPUT EVEN HANDLING (TO BE MOVED TO DIFFERENT LIBRARY)
//Input Event
#[derive(Clone, Copy)]
#[derive(Debug)]
#[repr(C)]
pub struct InputEvent {
    pub device_id:  u16,
    pub event_type: InputEventType,
    pub event_id:   u16,
    pub event_data: i16,
}

//Input Event Type
numeric_enum! {
    #[repr(u16)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    #[derive(PartialEq)]
    pub enum InputEventType {
        Blank              = 0x00,
        DigitalKey         = 0x01,
        DigitalButton      = 0x02,
        AnalogPosition     = 0x03,
        AnalogVelocity     = 0x04,
        AnalogAcceleration = 0x05,
    }
}

//PressType Enum
numeric_enum! {
    #[repr(i16)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum PressType {
        Press   = 1,
        Unpress = -1,
    }
}

//Physical Key ID Enum
numeric_enum! {
    #[repr(u16)]
    #[derive(Clone, Copy)]
    #[derive(Debug)]
    pub enum KeyID {
        //Standard Numeric Keys
        Key0                  = 0x0000, Key1                  = 0x0001, Key2                  = 0x0002, Key3                  = 0x0003, 
        Key4                  = 0x0004, Key5                  = 0x0005, Key6                  = 0x0006, Key7                  = 0x0007,
        Key8                  = 0x0008, Key9                  = 0x0009,
        //Standard Alphabet Keys
        KeyQ                  = 0x0010, KeyW                  = 0x0011, KeyE                  = 0x0012, KeyR                  = 0x0013,
        KeyT                  = 0x0014, KeyY                  = 0x0015, KeyU                  = 0x0016, KeyI                  = 0x0017,
        KeyO                  = 0x0018, KeyP                  = 0x0019, KeyA                  = 0x001A, KeyS                  = 0x001B,
        KeyD                  = 0x001C, KeyF                  = 0x001D, KeyG                  = 0x001E, KeyH                  = 0x001F,
        KeyJ                  = 0x0020, KeyK                  = 0x0021, KeyL                  = 0x0022, KeyZ                  = 0x0023,
        KeyX                  = 0x0024, KeyC                  = 0x0025, KeyV                  = 0x0026, KeyB                  = 0x0027,
        KeyN                  = 0x0028, KeyM                  = 0x0029,
        //Standard Punctuation Keys
        KeyGrave              = 0x0030, KeyDash               = 0x0031, KeyEqual              = 0x0032, KeyBackspace          = 0x0033, 
        KeyTab                = 0x0034, KeyOpenBracket        = 0x0035, KeyCloseBracket       = 0x0036, KeyBackSlash          = 0x0037, 
        KeySemicolon          = 0x0038, KeyQuote              = 0x0039, KeyEnter              = 0x003A, KeyComma              = 0x003B,
        KeyPeriod             = 0x003C, KeyForwardSlash       = 0x003D, KeySpace              = 0x003E,
        //Standard Modifier Keys
        KeyScrollLock         = 0x0040, KeyCapsLock           = 0x0041, KeyLeftShift          = 0x0042, KeyRightShift         = 0x0043,
        KeyLeftControl        = 0x0044, KeyRightControl       = 0x0045, KeyLeftAlt            = 0x0046, KeyRightAlt           = 0x0047,
        KeyLeftOperatingSystem                                = 0x0048, KeyRightOperatingSystem                               = 0x0049,
        //Standard Action Keys
        KeyEscape             = 0x0050, KeyPrintScreen        = 0x0051, KeyPause              = 0x0052, KeyHome               = 0x0053,
        KeyEnd                = 0x0054, KeyDelete             = 0x0055, KeyPageUp             = 0x0056, KeyPageDown           = 0x0057, 
        KeyInsert             = 0x0058, KeyUpArrow            = 0x0059, KeyDownArrow          = 0x005A, KeyLeftArrow          = 0x005B,
        KeyRightArrow         = 0x005C,
        //Standard Function Keys
        KeyF01                = 0x0060, KeyF02                = 0x0061, KeyF03                = 0x0062, KeyF04                = 0x0063,
        KeyF05                = 0x0064, KeyF06                = 0x0065, KeyF07                = 0x0066, KeyF08                = 0x0067,
        KeyF09                = 0x0068, KeyF10                = 0x0069, KeyF11                = 0x006A, KeyF12                = 0x006B,
        KeyF13                = 0x006C, KeyF14                = 0x006D, KeyF15                = 0x006E, KeyF16                = 0x007F,
        KeyF17                = 0x0070, KeyF18                = 0x0071, KeyF19                = 0x0072, KeyF20                = 0x0073,
        KeyF21                = 0x0074, KeyF22                = 0x0075, KeyF23                = 0x0076, KeyF24                = 0x0077,
        //Numpad Numeric Keys
        Num0                  = 0x0080, Num1                  = 0x0081, Num2                  = 0x0082, Num3                  = 0x0083,
        Num4                  = 0x0084, Num5                  = 0x0085, Num6                  = 0x0086, Num7                  = 0x0087,
        Num8                  = 0x0088, Num9                  = 0x0089,
        //Numpad Modifier Keys
        NumLock               = 0x0090,
        //Numpad Punctuation Keys
        NumForwardSlash       = 0x00A0, NumMultiply           = 0x00A1, NumMinus              = 0x00A2, NumPlus               = 0x00A3,
        NumEnter              = 0x00A4,
        //Media Keys
        MediaPlayPause        = 0x00B0, MediaNextTrack        = 0x00B1, MediaPreviousTrack    = 0x00B2, MediaStop             = 0x00B3, 
        MediaVolumeDown       = 0x00B4, MediaVolumeUp         = 0x00B5, MediaMute             = 0x00B6, MediaMyComputer       = 0x00B7, 
        MediaEmail            = 0x00B8, MediaSelect           = 0x00B9, MediaWebStop          = 0x00BA, MediaWebForward       = 0x00BB,
        MediaWebSearch        = 0x00BC, MediaWebBack          = 0x00BD, MediaWebHome          = 0x00BE, MediaWebFavorites     = 0x00BF, 
        MediaWebRefresh       = 0x00C0, MediaCalculator       = 0x00C1,
        //Non-standard Keys
        KeyNonUSPound         = 0xFF00, KeyDeleteForward      = 0xFF01, Num00                 = 0xFF02, Num000                = 0xFF03,
        NumPeriod             = 0xFF04, NumEqual              = 0xFF05, NumComma              = 0xFF06, NumEqualSign          = 0xFF07,
        NumOpenParenthesis    = 0xFF08, NumCloseParenthesis   = 0xFF09, NumOpenBrace          = 0xFF0A, NumCloseBrace         = 0xFF0B,
        NumTab                = 0xFF0C, NumBackspace          = 0xFF0D, NumA                  = 0xFF0E, NumB                  = 0xFF0F,
        NumC                  = 0xFF10, NumD                  = 0xFF11, NumE                  = 0xFF12, NumF                  = 0xFF13,
        NumExclusiveOr        = 0xFF14, NumExponent           = 0xFF15, NumPercent            = 0xFF16, NumLessThan           = 0xFF17,
        NumGreaterThan        = 0xFF18, NumLogicalAnd         = 0xFF19, NumBooleanAnd         = 0xFF1A, NumLogicalOr          = 0xFF1B,
        NumBooleanOr          = 0xFF1C, NumColon              = 0xFF1D, NumPound              = 0xFF1E, NumSpace              = 0xFF1F,
        NumAddress            = 0xFF20, NumNot                = 0xFF21, NumMemoryStore        = 0xFF22, NumMemoryRecall       = 0xFF23,
        NumMemoryClear        = 0xFF24, NumMemoryAdd          = 0xFF25, NumMemorySubtract     = 0xFF26, NumMemoryMultiply     = 0xFF27,
        NumMemoryDivide       = 0xFF28, NumPlusAndMinus       = 0xFF29, NumClear              = 0xFF2A, NumClearEntry         = 0xFF2B,
        NumBinary             = 0xFF2C, NumOctal              = 0xFF2D, NumDecimal            = 0xFF2E, NumHexadecimal        = 0xFF2F,
        KeyNonUSBackSlash     = 0xFF30, KeyApplication        = 0xFF31, KeyPower              = 0xFF32, KeySleep              = 0xFF33,
        KeyWake               = 0xFF34, KeyExecute            = 0xFF35, KeyHelp               = 0xFF36, KeyMenu               = 0xFF37,
        KeySelect             = 0xFF38, KeyStop               = 0xFF39, KeyAgain              = 0xFF3A, KeyUndo               = 0xFF3B,
        KeyCut                = 0xFF3C, KeyCopy               = 0xFF3D, KeyPaste              = 0xFF3E, KeyFind               = 0xFF3F,
        KeyCapsLockHold       = 0xFF40, KeyNumLockHold        = 0xFF41, KeyScrollLockHold     = 0xFF42, KeyInternational1     = 0xFF43,
        KeyInternational2     = 0xFF44, KeyInternational3     = 0xFF45, KeyInternational4     = 0xFF46, KeyInternational5     = 0xFF47,
        KeyInternational6     = 0xFF48, KeyInternational7     = 0xFF49, KeyInternational8     = 0xFF4A, KeyInternational9     = 0xFF4B,
        KeyLanguage1          = 0xFF4C, KeyLanguage2          = 0xFF4D, KeyLanguage3          = 0xFF4E, KeyLanguage4          = 0xFF4F,
        KeyLanguage5          = 0xFF50, KeyLanguage6          = 0xFF51, KeyLanguage7          = 0xFF52, KeyLanguage8          = 0xFF53,
        KeyLanguage9          = 0xFF54, KeyAlternateErase     = 0xFF55, KeyAttention          = 0xFF56, KeyCancel             = 0xFF57,
        KeyClear              = 0xFF58, KeyPrior              = 0xFF59, KeyReturn             = 0xFF5A, KeySeparator          = 0xFF5B,
        KeyOut                = 0xFF5C, KeyOperatingSystem    = 0xFF5D, KeyClearAgain         = 0xFF5E, KeyControlSelect      = 0xFF5F,
        KeyExecuteSelect      = 0xFF60, KeyThousandsSeparator = 0xFF61, KeyDecimalSeparator   = 0xFF62, KeyCurrencyUnit       = 0xFF63,
        KeyCurrencySubunit    = 0xFF64,
    }
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
        (KeyID::KeyQuote,        false, _) => KeyStr::Str("'"),  (KeyID::KeyQuote,        true, _) => KeyStr::Str("\""),
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

// PS/2 Scancode Translation Return Value
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum Ps2Scan {
    Finish(InputEvent),
    Continue,
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
