use core::fmt;
use std::path::PathBuf;

use crate::base24;
use crate::bytesize::BytesBase2;

#[derive(Debug)]
pub enum Error {
    FileRead { e: std::io::Error, path: PathBuf },
    GetSystemDirectory(windows::core::Error),
    Pe(editpe::ImageReadError),
    Base24(base24::error::Error),
    PeMissingResourceTableDataDirectory,
    PidgenDllUnexpectedSize { file_bytesize: BytesBase2 },
    PidgenBinkeyInvalidChecksum { result: u32 },
    PidgenBinkeyUnexpectedSize { cur: usize, exp: usize },
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileRead { e, path } => write!(f, "error reading file {:?}: {}", path, e),
            Self::GetSystemDirectory(e) => write!(f, "error finding system directory: {}", e),
            Self::Pe(e) => write!(f, "invalid executable file: {}", e),
            Self::Base24(e) => write!(f, "base24 encode/decode error: {}", e),
            Self::PeMissingResourceTableDataDirectory => f.write_str("executable file missing data directory for the resource table"),
            Self::PidgenDllUnexpectedSize { file_bytesize } => write!(f, "pidgen DLL file has unexpected size ({} > 1 MiB)", file_bytesize),
            Self::PidgenBinkeyInvalidChecksum { result } => write!(f, "invalid BINKEY checksum: 0x{:08x} != 0", result),
            Self::PidgenBinkeyUnexpectedSize { cur, exp } => write!(f, "BINKEY structure has unexpected size {} < {}", BytesBase2::from_bytes(*cur as f64), BytesBase2::from_bytes(*exp as f64)),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
