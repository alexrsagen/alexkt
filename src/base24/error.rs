use core::fmt;

#[derive(Debug)]
pub enum Error {
    AlphabetLengthInvalid,
    EncodeInputLengthInvalid,
    DecodeInputLengthInvalid,
    DecodeUnsupportedCharacter(char),
    DecodeInvalidEncoding,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlphabetLengthInvalid => write!(f, "Alphabet length must be less than {}", u32::MAX),
            Self::EncodeInputLengthInvalid => write!(f, "Input data length must be a multiple of 4 bytes (32 bits)"),
            Self::DecodeInputLengthInvalid => write!(f, "Input data length must be a multiple of 7 chars"),
            Self::DecodeUnsupportedCharacter(c) => write!(f, "Unsupported character in input: {0:?}", c),
            Self::DecodeInvalidEncoding => write!(f, "Input data is not a valid encoding with this alphabet"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
