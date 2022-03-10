// GLUON: NOBLE RETURN CODE


// HEADER
//Imports
//use core::ops::FromResidual;
//use core::ops::Try;
//use core::ops::ControlFlow;


// RETURN CODE
#[repr(u64)]
#[derive(Debug)]
pub enum ReturnCode {
    NoError               = 0x00,
    UnknownError          = 0x01,
    MemoryOutOfBounds     = 0x02,
    VolumeOutOfBounds     = 0x03,
    BufferTooSmall        = 0x04,
    InvalidIdentifier     = 0x05,
    UnsupportedFeature    = 0x06,
    InvalidData           = 0x07,
    IncompleteAccess      = 0x08,
    IncorrectBufferLength = 0x09,
    SlicingError          = 0x0A,
    IndexOutOfBounds      = 0x0B,
    ConversionError       = 0x0C,
    BufferTooLarge        = 0x0D,
    SeekError             = 0x0E,
    NotYetImplemented     = 0x0F,
    UefiError             = 0x10,
    UnknownGlyph          = 0x11,
    FileDeleteFailure     = 0x12,
    ReadError             = 0x13,
    TimeOut               = 0x14,
    IncompatibleVersion   = 0x15,
    InvalidLanguage       = 0x16,
    CompromisedData       = 0x17,
    WriteFailure          = 0x18,
    StaleData             = 0x19,
    FileSystemDump        = 0x1A,
    ResetRequested        = 0x1B,
    NotReady              = 0x1C,
    DeviceError           = 0x1D,
    WriteProtected        = 0x1E,
    OutOfResources        = 0x1F,
    VolumeCorrupted       = 0x20,
    VolumeFull            = 0x21,
    MediaMissing          = 0x22,
    MediaChanged          = 0x23,
    NotFound              = 0x24,
    AccessDenied          = 0x25,
    NoResponse            = 0x26,
    NoMapping             = 0x27,
    NotStarted            = 0x28,
    AlreadyStarted        = 0x29,
    Aborted               = 0x2A,
    IcmpError             = 0x2B,
    TftpError             = 0x2C,
    ProtocolError         = 0x2D,
    SecurityViolation     = 0x2E,
    CrcError              = 0x2F,
    EndOfVolume           = 0x30,
    AddressConflict       = 0x32,
    HttpError             = 0x33,
    InvalidCharacter      = 0x34,
    DataTooLarge          = 0x35,
    DirectoryFull         = 0x36,
    NotPresent            = 0x37,
    Test00                = 0xFFFF_FFFF_FFFF_FF00,
    Test01                = 0xFFFF_FFFF_FFFF_FF01,
    Test02                = 0xFFFF_FFFF_FFFF_FF02,
    Test03                = 0xFFFF_FFFF_FFFF_FF03,
    Test04                = 0xFFFF_FFFF_FFFF_FF04,
    Test05                = 0xFFFF_FFFF_FFFF_FF05,
    Test06                = 0xFFFF_FFFF_FFFF_FF06,
    Test07                = 0xFFFF_FFFF_FFFF_FF07,
    Test08                = 0xFFFF_FFFF_FFFF_FF08,
    Test09                = 0xFFFF_FFFF_FFFF_FF09,
}

/*impl ReturnCode {
    pub fn as_result(self) -> Result<(), ReturnCode> {
        match self {
            ReturnCode::NoError => Ok(()),
            error => Err(self)
        }
    }
}
impl Try for ReturnCode {
    type Output = ();

    type Residual = Self;

    fn from_output(output: Self::Output) -> Self {
        Self::NoError
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            ReturnCode::NoError => ControlFlow::Continue(()),
            _ => ControlFlow::Break(self),
        }
    }
}
impl FromResidual for ReturnCode {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        residual
    }
}
*/
