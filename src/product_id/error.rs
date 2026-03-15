use core::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ProductIdError {
    TooShort,
    InvalidCharacter,
}

impl core::error::Error for ProductIdError {}

impl fmt::Display for ProductIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
			Self::TooShort => write!(f, "Product ID is too short."),
			Self::InvalidCharacter => write!(f, "Invalid character in product ID."),
		}
    }
}
