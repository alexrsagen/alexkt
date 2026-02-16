use core::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ConfirmationIdError {
    TooShort,
    TooLong,
    InvalidCharacter,
    InvalidCheckDigit,
    UnknownVersion,
    UnspecifiedProductID,
    Unlucky,
}

impl core::error::Error for ConfirmationIdError {}

impl fmt::Display for ConfirmationIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
			&Self::TooShort => write!(f, "Installation ID is too short."),
			&Self::TooLong => write!(f, "Installation ID is too long."),
			&Self::InvalidCharacter => write!(f, "Invalid character in installation ID."),
			&Self::InvalidCheckDigit => write!(f, "Installation ID checksum failed. Please check that it is typed correctly."),
			&Self::UnknownVersion => write!(f, "Unknown installation ID version."),
			&Self::UnspecifiedProductID => write!(f, "No product ID specified."),
			&Self::Unlucky => write!(f, "Unable to generate valid confirmation ID."),
		}
    }
}