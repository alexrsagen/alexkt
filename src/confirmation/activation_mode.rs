use core::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseActivationModeError;

impl core::error::Error for ParseActivationModeError {}

impl fmt::Display for ParseActivationModeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("attempted to convert a string that doesn't match an existing activation mode")
    }
}

#[repr(usize)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ActivationMode {
    #[default]
    Windows,
    OfficeXP,
    Office2003,
    Office2007,
    PlusDigitalMediaEdition,
}

impl ActivationMode {
    const MODE_NAMES: [&'static str; 5] = [
        "Windows",
        "OfficeXP",
        "Office2003",
        "Office2007",
        "PlusDigitalMediaEdition",
    ];

    fn from_usize(u: usize) -> Option<Self> {
        match u {
            0 => Some(Self::Windows),
            1 => Some(Self::OfficeXP),
            2 => Some(Self::Office2003),
            3 => Some(Self::Office2007),
            4 => Some(Self::PlusDigitalMediaEdition),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        Self::MODE_NAMES[*self as usize]
    }

    pub fn is_windows(&self) -> bool {
        match self {
            Self::Windows | Self::PlusDigitalMediaEdition => true,
            _ => false,
        }
    }

    pub fn is_office(&self) -> bool {
        match self {
            Self::Office2003 | Self::Office2007 | Self::OfficeXP => true,
            _ => false,
        }
    }

    pub fn installation_id_key(&self) -> [u8; 4] {
        if self.is_office() {
            [0x5A, 0x30, 0xB9, 0xF3]
        } else {
            [0x6A, 0xC8, 0x5E, 0xD4]
        }
    }
}

impl FromStr for ActivationMode {
    type Err = ParseActivationModeError;
    fn from_str(level: &str) -> std::result::Result<ActivationMode, Self::Err> {
        Self::MODE_NAMES
            .iter()
            .position(|&name| name.eq_ignore_ascii_case(level))
            .map(|p| ActivationMode::from_usize(p))
            .flatten()
            .ok_or(ParseActivationModeError)
    }
}

impl fmt::Display for ActivationMode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(self.as_str())
    }
}
