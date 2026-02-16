use core::fmt;

use bitreader::BitReaderError;

#[derive(Debug)]
pub enum Error {
    InvalidParameters,
	InvalidKeyFormat,
	InvalidKey,
	BitReaderError(BitReaderError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters => write!(f, "Invalid curve/point parameters"),
            Self::InvalidKeyFormat => write!(f, "Invalid serial key format"),
            Self::InvalidKey => write!(f, "Invalid serial key (could not be verified)"),
			Self::BitReaderError(e) => write!(f, "Invalid data: {e}"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<BitReaderError> for Error {
	fn from(value: BitReaderError) -> Self {
		Self::BitReaderError(value)
	}
}
