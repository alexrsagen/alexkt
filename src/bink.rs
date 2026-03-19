use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use editpe::{Image, ResourceDirectory, ResourceEntryName};
use log::debug;
use num_bigint::BigInt;

use crate::bytesize::{BytesBase2, MIB};
use crate::error::{Error, Result};
use crate::crypto::{EllipticCurve, PublicKey, WithCurve, WithPublicKey};
use crate::system_directory::get_system_directory;

pub fn get_binkey_resources_from_image(image: &Image) -> Result<Vec<Binkey>> {
    let mut binkeys = Vec::new();

    let resource_table = get_resources_from_image(&image)?;

    let maybe_bink = resource_table
        .root()
        .get(ResourceEntryName::from_string("BINK"))
        .map(|e| e.as_table())
        .flatten();

    if let Some(bink) = maybe_bink {
        debug!("found BINK resource table: {:#x?}", bink);

        for entry_name in bink.entries() {
            let maybe_bink_entry = bink.get(entry_name).map(|e| e.as_table()).flatten();

            if let Some(bink_entry) = maybe_bink_entry {
                debug!("found BINK entry table: {:#x?}", bink_entry);

                for bink_entry_name in bink_entry.entries() {
                    let maybe_bink_data = bink_entry
                        .get(bink_entry_name)
                        .map(|e| e.as_data())
                        .flatten();

                    if let Some(bink_data) = maybe_bink_data {
                        let binkey = Binkey::parse(bink_data.data())?;
                        binkeys.push(binkey);
                    }
                }
            }
        }
    }

    Ok(binkeys)
}

pub fn get_binkey_resources_from_bytes(bytes: &[u8]) -> Result<Vec<Binkey>> {
    if is_nt_image(&bytes) {
        debug!("input data is an NT image");
        let image = get_image_from_bytes(bytes)?;
        get_binkey_resources_from_image(&image)
            .map(|binkeys| binkeys.into_iter().map(|k| k.to_owned()).collect())
    } else {
        debug!("input data is not an NT image, likely a BINK resource");
        let binkey = Binkey::parse(bytes)?;
        Ok(vec![binkey])
    }
}

pub fn get_binkey_resources_from_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<Binkey>> {
    let mut file = File::open(&file_path).map_err(|e| Error::FileRead { e, path: file_path.as_ref().into() })?;
    let file_len = file.metadata().map_err(|e| Error::FileRead { e, path: file_path.as_ref().into() })?.len();
    let file_bytesize = BytesBase2::from_bytes(file_len as f64);

    debug!(
        "opened file for reading: {:?} ({})",
        file_path.as_ref(),
        file_bytesize
    );

    if file_len > MIB {
        return Err(Error::PidgenDllUnexpectedSize { file_bytesize });
    }

    let mut file_contents = Vec::with_capacity(file_len as usize);
    file.read_to_end(&mut file_contents).map_err(|e| Error::FileRead { e, path: file_path.as_ref().into() })?;

    get_binkey_resources_from_bytes(&file_contents)
}

pub fn get_binkey_resources_from_system_file() -> Result<Vec<Binkey>> {
    let system32_path = get_system_directory().map_err(Error::GetSystemDirectory)?;
    let system_file_path = PathBuf::from(&system32_path).join("pidgen.dll");
    get_binkey_resources_from_file(&system_file_path)
}

/// File offset to IMAGE_DOS_HEADER.e_lfanew,
/// which is a file offset to IMAGE_NT_HEADERS.Signature
const E_LFANEW_OFFSET: usize = 0x3c;

fn is_nt_image(bytes: &[u8]) -> bool {
    if !is_dos_image(bytes) {
        return false;
    }

    let nt_header_offset = u32::from_le_bytes([
        bytes[E_LFANEW_OFFSET],
        bytes[E_LFANEW_OFFSET + 1],
        bytes[E_LFANEW_OFFSET + 2],
        bytes[E_LFANEW_OFFSET + 3],
    ]) as usize;

    if bytes.len() < nt_header_offset + 4 {
        return false;
    }

    bytes[nt_header_offset] == b'P'
        && bytes[nt_header_offset + 1] == b'E'
        && bytes[nt_header_offset + 2] == 0x00
        && bytes[nt_header_offset + 3] == 0x00
}

fn is_dos_image(bytes: &[u8]) -> bool {
    if bytes.len() < E_LFANEW_OFFSET + 4 {
        return false;
    }

    bytes[0] == b'M' && bytes[1] == b'Z'
}

fn get_image_from_bytes<'a>(bytes: &'a [u8]) -> Result<Image<'a>> {
    Image::parse(bytes).map_err(Error::Pe)
}

fn get_resources_from_image<'a>(image: &'a Image) -> Result<&'a ResourceDirectory> {
    if let Some(resources) = image.resource_directory() {
        Ok(resources)
    } else {
        Err(Error::PeMissingResourceTableDataDirectory)
    }
}

#[derive(Debug, Clone)]
pub struct BinkeyHeader {
    pub header_words: u32,
    pub checksum: u32,
    pub version: u32,
    pub key_words: u32,
    pub hash_len: u32,
    pub sig_len: u32,
    pub authlen: Option<u32>,
    pub pidlen: Option<u32>,
    pub extra: Option<Vec<u32>>,
}

impl BinkeyHeader {
    pub fn version_date(&self) -> Option<NaiveDate> {
        let year = (self.version / 10000) as i32;
        let month = self.version / 100 % 100;
        let day = self.version % 100;
        NaiveDate::from_ymd_opt(year, month, day)
    }

    pub fn key_len(&self) -> usize {
        self.key_words as usize * 4
    }

    pub fn key_bits(&self) -> usize {
        self.key_len() * 8
    }
}

#[derive(Debug, Clone)]
pub struct Binkey {
    pub id: u32,
    pub header: BinkeyHeader,
    pub curve: EllipticCurve,
    pub public: PublicKey,
}

impl WithCurve for Binkey {
    fn curve(&self) -> &EllipticCurve {
        &self.curve
    }
}

impl WithPublicKey for Binkey {
    fn public_key(&self) -> &PublicKey {
        &self.public
    }
}

impl Binkey {
    const MIN_HEADER_SIZE: usize = 32;
    const EXT_HEADER_SIZE: usize = 40;
    const PUBKEY_FIELD_COUNT: usize = 7;

    /// Returns the size of the BINKEY pubkey fields, in bytes
    pub fn fields_len(&self) -> usize {
        self.header.key_len() * Self::PUBKEY_FIELD_COUNT
    }

    /// Returns the size of the BINKEY structure, in bytes
    pub fn len(&self) -> usize {
        self.header.header_words as usize * 4 + self.fields_len()
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let bytes_len = bytes.len();
        if bytes_len < Self::MIN_HEADER_SIZE || bytes_len % 4 != 0 || bytes_len > MIB as usize {
            return Err(Error::PidgenBinkeyUnexpectedSize {
                cur: bytes_len,
                exp: Self::MIN_HEADER_SIZE,
            });
        }

        // read BINKEY ID (not part of checksum)
        let (id_bytes, bytes) = bytes.split_at(4);
        let id = u32::from_le_bytes(id_bytes.try_into().unwrap());
        debug!("parsing BINKEY ID 0x{:02X}", id);

        // read all words of BINKEY structure
        let word_count = bytes_len / 4 - 1;
        let mut words = Vec::<u32>::with_capacity(word_count);
        for i in 0..word_count {
            let start = i * 4;
            let end = start + 4;
            words.push(u32::from_le_bytes(bytes[start..end].try_into().unwrap()));
        }

        // verify checksum
        let mut result = 0u32;
        for word in &words {
            (result, _) = result.overflowing_add(*word);
        }
        if result != 0 {
            return Err(Error::PidgenBinkeyInvalidChecksum { result });
        }

        let expected_bytes_len = words[0] as usize + 4;
        if expected_bytes_len != bytes_len {
            return Err(Error::PidgenBinkeyUnexpectedSize {
                cur: bytes_len,
                exp: expected_bytes_len,
            });
        }

        let header_words = words[1];

        let authlen = if header_words > 7 {
            Some(words[7])
        } else {
            None
        };

        let pidlen = if header_words > 8 {
            Some(words[8])
        } else {
            None
        };

        let checksum = words[2];
        let version = words[3];
        let key_words = words[4];
        let hash_len = words[5];
        let sig_len = words[6];

        let extra = if header_words > 9 {
            Some(words[9..header_words as usize].to_vec())
        } else {
            None
        };

        let header = BinkeyHeader {
            header_words,
            checksum,
            version,
            key_words,
            hash_len,
            sig_len,
            authlen,
            pidlen,
            extra,
        };

        let key_words = header.key_words as usize;
        let mut offset = header_words as usize;
        let p = BigInt::from_slice(num_bigint::Sign::Plus, &words[offset..offset+key_words]);
        offset += key_words;
        let a = BigInt::from_slice(num_bigint::Sign::Plus, &words[offset..offset+key_words]);
        offset += key_words;
        let b = BigInt::from_slice(num_bigint::Sign::Plus, &words[offset..offset+key_words]);
        offset += key_words;
        let gx = BigInt::from_slice(num_bigint::Sign::Plus, &words[offset..offset+key_words]);
        offset += key_words;
        let gy = BigInt::from_slice(num_bigint::Sign::Plus, &words[offset..offset+key_words]);
        offset += key_words;
        let kx = BigInt::from_slice(num_bigint::Sign::Plus, &words[offset..offset+key_words]);
        offset += key_words;
        let ky = BigInt::from_slice(num_bigint::Sign::Plus, &words[offset..offset+key_words]);

        let curve = EllipticCurve::new(p, a, b);
        let pubkey = PublicKey::new(gx, gy, kx, ky);

        let binkey = Self {
            id,
            header,
            curve,
            public: pubkey,
        };

        debug!("parsed BINKEY: {:#X?}", binkey);
        Ok(binkey)
    }
}

impl PartialEq for Binkey {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}
