use crate::product_id::error::ProductIdError;

use core::fmt;
use std::string::FromUtf8Error;

#[derive(Debug, PartialEq, Eq)]
pub enum InstallationIdError {
    TooShort,
    TooLong,
    InvalidCharacter,
    InvalidCheckDigit,
    UnknownVersion,
    UnspecifiedProductID,
}

impl core::error::Error for InstallationIdError {}

impl fmt::Display for InstallationIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
			Self::TooShort => write!(f, "Installation ID is too short."),
			Self::TooLong => write!(f, "Installation ID is too long."),
			Self::InvalidCharacter => write!(f, "Invalid character in installation ID."),
			Self::InvalidCheckDigit => write!(f, "Installation ID checksum failed. Please check that it is typed correctly."),
			Self::UnknownVersion => write!(f, "Unknown installation ID version."),
			Self::UnspecifiedProductID => write!(f, "No product ID specified."),
		}
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfirmationIdError {
    FromUtf8Error(FromUtf8Error),
    InstallationIdError(InstallationIdError),
    ProductIdError(ProductIdError),
    Unlucky,
}

impl core::error::Error for ConfirmationIdError {}

impl fmt::Display for ConfirmationIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FromUtf8Error(e) => write!(f, "Confirmation ID contains invalid UTF-8 characters: {e}"),
			Self::InstallationIdError(e) => write!(f, "Unable to parse installation ID: {e}"),
			Self::ProductIdError(e) => write!(f, "Unable to parse product ID: {e}"),
			Self::Unlucky => write!(f, "Unable to generate valid confirmation ID."),
		}
    }
}

impl From<FromUtf8Error> for ConfirmationIdError {
    fn from(inner: FromUtf8Error) -> Self {
        Self::FromUtf8Error(inner)
    }
}

impl From<InstallationIdError> for ConfirmationIdError {
    fn from(inner: InstallationIdError) -> Self {
        Self::InstallationIdError(inner)
    }
}

impl From<ProductIdError> for ConfirmationIdError {
    fn from(inner: ProductIdError) -> Self {
        Self::ProductIdError(inner)
    }
}
