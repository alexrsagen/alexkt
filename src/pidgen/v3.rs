use core::fmt;
use std::str::FromStr;

use bitreader::BitReader;
use log::debug;
use num_bigint::{BigInt, BigUint, RandomBits};
use num_integer::Integer;
use num_traits::ToPrimitive;
use rand::{Rng, RngExt};
use sha1_smol::Sha1;

use super::error::{Error, Result};
use crate::base24::Base24;
use crate::crypto::{mod_sqrt, Point, WithCurve, WithPrivateKey, WithPublicKey};

const UPGRADE_LENGTH_BITS: u8 = 1;
const MAX_RETRIES: usize = 10;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseKeyVersionError;

impl core::error::Error for ParseKeyVersionError {}

impl fmt::Display for ParseKeyVersionError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("attempted to convert a string that doesn't match an existing key version")
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyVersion {
    #[default]
    Bink1998,
    Bink2002,
}

impl KeyVersion {
    const VERSION_NAMES: [&'static str; 2] = ["Bink1998", "Bink2002"];

    fn from_usize(u: usize) -> Option<Self> {
        match u {
            0 => Some(Self::Bink1998),
            1 => Some(Self::Bink2002),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        Self::VERSION_NAMES[*self as usize]
    }

    pub const fn max_seqauth(self) -> u32 {
        match self {
            KeyVersion::Bink1998 => 999999,
            KeyVersion::Bink2002 => 1023,
        }
    }

    pub const fn sig_bits(self) -> u8 {
        match self {
            KeyVersion::Bink1998 => 55,
            KeyVersion::Bink2002 => 62,
        }
    }

    pub const fn hash_bits(self) -> u8 {
        match self {
            KeyVersion::Bink1998 => 28,
            KeyVersion::Bink2002 => 31,
        }
    }

    pub const fn channel_id_bits(self) -> u8 {
        match self {
            KeyVersion::Bink1998 => 0,
            KeyVersion::Bink2002 => 10,
        }
    }

    pub const fn serial_bits(self) -> u8 {
        match self {
            KeyVersion::Bink1998 => 30,
            KeyVersion::Bink2002 => 0,
        }
    }

    pub const fn field_bits(self) -> u64 {
        match self {
            KeyVersion::Bink1998 => 384,
            KeyVersion::Bink2002 => 512,
        }
    }

    pub const fn other_bits(self) -> u8 {
        match self {
            KeyVersion::Bink1998 => self.hash_bits() + self.serial_bits() + UPGRADE_LENGTH_BITS,
            KeyVersion::Bink2002 => {
                self.sig_bits() + self.hash_bits() + self.channel_id_bits() + UPGRADE_LENGTH_BITS
            }
        }
    }

    pub const fn field_bytes(self) -> usize {
        (self.field_bits() as usize).div_ceil(8)
    }

    pub const fn sha_msg_length(self) -> usize {
        match self {
            KeyVersion::Bink1998 => 4 + 2 * self.field_bytes(),
            KeyVersion::Bink2002 => 3 + 2 * self.field_bytes(),
        }
    }
}

impl FromStr for KeyVersion {
    type Err = ParseKeyVersionError;
    fn from_str(level: &str) -> std::result::Result<KeyVersion, Self::Err> {
        Self::VERSION_NAMES
            .iter()
            .position(|&name| name.eq_ignore_ascii_case(level))
            .map(|p| KeyVersion::from_usize(p))
            .flatten()
            .ok_or(ParseKeyVersionError)
    }
}

impl fmt::Display for KeyVersion {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsignedProductKey {
    version: KeyVersion,
    channel_id: u32,
    sequence_or_authinfo: u32,
    upgrade: bool,
}

impl UnsignedProductKey {
    pub fn new<R: Rng>(version: KeyVersion, channel_id: u32, upgrade: bool, rng: &mut R) -> Self {
        let sequence_or_authinfo = rng.random_range(0..version.max_seqauth());
        Self::new_with_seqauth(version, channel_id, sequence_or_authinfo, upgrade)
    }

    pub fn new_with_seqauth(
        version: KeyVersion,
        channel_id: u32,
        sequence_or_authinfo: u32,
        upgrade: bool,
    ) -> Self {
        Self {
            version,
            channel_id,
            sequence_or_authinfo,
            upgrade,
        }
    }

    fn hash(&self, point: Point) -> Result<u32> {
        let Point::Point { x, y } = point else {
            return Err(Error::InvalidParameters);
        };

        let x_bin = x.to_bytes_le().1;
        if x_bin.len() > self.version.field_bytes() {
            return Err(Error::InvalidParameters);
        }

        let y_bin = y.to_bytes_le().1;
        if y_bin.len() > self.version.field_bytes() {
            return Err(Error::InvalidParameters);
        }

        let data = match self.version {
            KeyVersion::Bink1998 => {
                let serial = self.channel_id * 1_000_000 + self.sequence_or_authinfo;
                serial << 1 | self.upgrade as u32
            }
            KeyVersion::Bink2002 => self.channel_id << 1 | self.upgrade as u32,
        };

        let mut msg_buffer = vec![0; self.version.sha_msg_length()];
        match self.version {
            KeyVersion::Bink1998 => {
                msg_buffer[0..4].copy_from_slice(&data.to_le_bytes());
                msg_buffer[4..4 + x_bin.len()].copy_from_slice(&x_bin);
                msg_buffer
                    [4 + self.version.field_bytes()..4 + self.version.field_bytes() + y_bin.len()]
                    .copy_from_slice(&y_bin);
            }
            KeyVersion::Bink2002 => {
                msg_buffer[0x00] = 0x79;
                msg_buffer[1..3].copy_from_slice(&data.to_le_bytes()[0..2]);
                msg_buffer[3..3 + x_bin.len()].copy_from_slice(&x_bin);
                msg_buffer
                    [3 + self.version.field_bytes()..3 + self.version.field_bytes() + y_bin.len()]
                    .copy_from_slice(&y_bin);
            }
        }

        let msg_digest = {
            let mut hasher = Sha1::new();
            hasher.update(msg_buffer.as_slice());
            hasher.digest().bytes()
        };

        let digest_num = u32::from_le_bytes(msg_digest[0..4].try_into().unwrap());
        match self.version {
            KeyVersion::Bink1998 => Ok((digest_num >> 4) & 0xfffffff),
            KeyVersion::Bink2002 => Ok(digest_num & 0x7fffffff),
        }
    }

    fn e(&self, hash: u32) -> BigInt {
        match self.version {
            KeyVersion::Bink1998 => BigInt::from(hash),
            KeyVersion::Bink2002 => {
                let data = self.channel_id << 1 | self.upgrade as u32;

                let mut msg_buffer = [0; 11];
                msg_buffer[0] = 0x5D;
                msg_buffer[1..3].copy_from_slice(&data.to_le_bytes()[0..2]);
                msg_buffer[3..7].copy_from_slice(&hash.to_le_bytes());
                msg_buffer[7..9].copy_from_slice(&self.sequence_or_authinfo.to_le_bytes()[0..2]);

                let msg_digest = {
                    let mut hasher = Sha1::new();
                    hasher.update(msg_buffer.as_slice());
                    hasher.digest().bytes()
                };

                let digest_num1 = u32::from_le_bytes(msg_digest[0..4].try_into().unwrap()) as u64;
                let digest_num2 = u32::from_le_bytes(msg_digest[4..8].try_into().unwrap()) as u64;
                let i_signature = (((digest_num2 >> 2) & 0x3fffffff) << u32::BITS) | digest_num1;

                BigInt::from(i_signature)
            }
        }
    }

    fn signature<T: WithCurve + WithPublicKey + WithPrivateKey>(
        &self,
        hash: u32,
        seed: &BigInt,
        ckp: &T,
    ) -> Result<u64> {
        match self.version {
            KeyVersion::Bink1998 => {
                let mut ek = ckp.private_key().private();
                ek *= hash;

                let s = (ek + seed).mod_floor(&ckp.private_key().n);

                s.to_u64().ok_or(Error::InvalidParameters)
            }

            KeyVersion::Bink2002 => {
                let ek = ckp.private_key().private();
                let mut e = self.e(hash);
                e = (e * ek).mod_floor(&ckp.private_key().n);

                let mut s = e.clone();
                s = (&s * &s).mod_floor(&ckp.private_key().n);
                s = &s + seed;

                let Some(mut s) = mod_sqrt(&s, &ckp.private_key().n) else {
                    return Err(Error::InvalidParameters);
                };

                s = (s - e).mod_floor(&ckp.private_key().n);
                if s.is_odd() {
                    s = &s + &ckp.private_key().n;
                }
                s >>= 1;

                s.to_u64().ok_or(Error::InvalidParameters)
            }
        }
    }
}

/// A product key for the PIDGEN3 system
/// (supports both BINK1998 and BINK2002)
///
/// Every `ProductKey` contains a valid key for its given parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProductKey {
    inner: UnsignedProductKey,
    hash: u32,
    signature: u64,
}

impl ProductKey {
    /// Validates an existing product key string and tries to create a new `ProductKey` from it.
    ///
    /// # Arguments
    ///
    /// * `curve` - The elliptic curve to use for verification.
    /// * `key` - Should be 25 characters long, not including the (optional) hyphens.
    pub fn from_key<T: WithCurve + WithPublicKey>(
        key: &str,
        cp: &T,
        b24: &Base24,
        version: KeyVersion,
    ) -> Result<Self> {
        let packed_key = b24
            .decode_to_biguint(key, true)
            .map_err(|_| Error::InvalidKeyFormat)?;
        let product_key = Self::from_packed(&packed_key, version)?;
        if product_key.verify(cp)? {
            Ok(product_key)
        } else {
            Err(Error::InvalidKey)
        }
    }

    fn e(&self) -> BigInt {
        self.inner.e(self.hash)
    }

    fn sig_point<T: WithCurve + WithPublicKey>(&self, cp: &T) -> Point {
        let e = self.e();
        let s = BigInt::from(self.signature);
        let t = cp.curve().multiply_point(&s, &cp.public_key().g);
        let p = cp.curve().multiply_point(&e, &cp.public_key().k);
        let p = cp.curve().add_points(&p, &t);

        match self.inner.version {
            KeyVersion::Bink1998 => p,
            KeyVersion::Bink2002 => cp.curve().multiply_point(&s, &p),
        }
    }

    /// Generates a new product key for the given parameters.
    ///
    /// The generated key is guaranteed to be valid.
    pub fn generate<T: WithCurve + WithPrivateKey + WithPublicKey, R: Rng>(
        unsigned_product_key: UnsignedProductKey,
        rng: &mut R,
        ckp: &T,
    ) -> Result<Self> {
        for _ in 0..MAX_RETRIES {
            let seed: BigUint =
                rng.sample(RandomBits::new(unsigned_product_key.version.field_bits()));
            let mut seed: BigInt = seed.into();

            let r = ckp.curve().multiply_point(&seed, &ckp.public_key().g);
            let hash = match unsigned_product_key.hash(r) {
                Ok(hash) => hash,
                Err(Error::InvalidParameters) => {
                    debug!("unable to generate hash (x or y too big?), retrying...");
                    continue;
                }
                Err(e) => return Err(e),
            };

            if unsigned_product_key.version == KeyVersion::Bink2002 {
                seed <<= 2;
            }

            let signature = match unsigned_product_key.signature(hash, &seed, ckp) {
                Ok(hash) => hash,
                Err(Error::InvalidParameters) => {
                    debug!("unable to generate signature, retrying...");
                    continue;
                }
                Err(e) => return Err(e),
            };

            if signature <= (1 << unsigned_product_key.version.sig_bits() as u64) - 1 {
                return Ok(Self {
                    inner: unsigned_product_key,
                    hash,
                    signature,
                });
            }
        }

		Err(Error::InvalidParameters)
    }

    fn verify<T: WithCurve + WithPublicKey>(&self, cp: &T) -> Result<bool> {
        let p = self.sig_point(cp);
        let hash = self.inner.hash(p)?;
        Ok(hash == self.hash)
    }

    fn from_packed(packed_key: &BigUint, version: KeyVersion) -> Result<Self> {
        let packed_key = packed_key.to_bytes_be();
        let mut reader = BitReader::new(&packed_key);

        // The signature/authinfo length isn't known (depending on version),
        // but everything else is, so we can calculate it
        let remaining_bits = ((packed_key.len() * 8) as u8)
            .checked_sub(version.other_bits())
            .ok_or(Error::InvalidKeyFormat)?;

        let key = match version {
            KeyVersion::Bink1998 => {
                let signature = reader.read_u64(remaining_bits)?;
                let hash = reader.read_u32(version.hash_bits())?;
                let serial = reader.read_u32(version.serial_bits())?;
                let upgrade = reader.read_bool()?;
                let sequence_or_authinfo = serial % 1_000_000;
                let channel_id = serial / 1_000_000;
                Self {
                    inner: UnsignedProductKey {
                        version,
                        channel_id,
                        sequence_or_authinfo,
                        upgrade,
                    },
                    hash,
                    signature,
                }
            }

            KeyVersion::Bink2002 => {
                let sequence_or_authinfo = reader.read_u32(remaining_bits)?;
                let signature = reader.read_u64(version.sig_bits())?;
                let hash = reader.read_u32(version.hash_bits())?;
                let channel_id = reader.read_u32(version.channel_id_bits())?;
                let upgrade = reader.read_bool()?;
                Self {
                    inner: UnsignedProductKey {
                        version,
                        channel_id,
                        sequence_or_authinfo,
                        upgrade,
                    },
                    hash,
                    signature,
                }
            }
        };

        Ok(key)
    }

    fn pack(&self) -> BigUint {
        let mut packed_key: u128 = 0;

        match self.inner.version {
            KeyVersion::Bink1998 => {
                let serial = self.inner.channel_id * 1_000_000 + self.inner.sequence_or_authinfo;
                packed_key |= (self.signature as u128) << self.inner.version.other_bits();
                packed_key |=
                    (self.hash as u128) << (self.inner.version.serial_bits() + UPGRADE_LENGTH_BITS);
                packed_key |= (serial as u128) << UPGRADE_LENGTH_BITS;
            }

            KeyVersion::Bink2002 => {
                packed_key |=
                    (self.inner.sequence_or_authinfo as u128) << self.inner.version.other_bits();
                packed_key |= (self.signature as u128)
                    << (self.inner.version.other_bits() - self.inner.version.sig_bits());
                packed_key |= (self.hash as u128)
                    << (self.inner.version.channel_id_bits() + UPGRADE_LENGTH_BITS);
                packed_key |= (self.inner.channel_id as u128) << UPGRADE_LENGTH_BITS;
            }
        }

        packed_key |= self.inner.upgrade as u128;

        BigUint::from_bytes_be(&packed_key.to_be_bytes())
    }

    /// Returns the `upgrade` field encoded in the key
    pub fn is_upgrade(&self) -> bool {
        self.inner.upgrade
    }

    /// Returns the `channel_id` field encoded in the key
    pub fn get_channel_id(&self) -> u32 {
        self.inner.channel_id
    }

    /// Returns the `sequence` field encoded in the key
    pub fn get_sequence(&self) -> u32 {
        self.inner.sequence_or_authinfo
    }

    /// Returns the `hash` field encoded in the key
    pub fn get_hash(&self) -> u32 {
        self.hash
    }

    /// Returns the `signature` field encoded in the key
    pub fn get_signature(&self) -> u64 {
        self.signature
    }
}

impl fmt::Display for ProductKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b24 = Base24::with_alphabet(Base24::ALPHABET_MS).map_err(|_| fmt::Error)?;
        let pk = b24.encode_biguint(&self.pack());
        let key = pk
            .chars()
            .enumerate()
            .fold(String::new(), |mut acc: String, (i, c)| {
                if i > 0 && i % 5 == 0 {
                    acc.push('-');
                }
                acc.push(c);
                acc
            });
        write!(f, "{}", key)
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyVersion, ProductKey};
    use crate::base24::Base24;
    use crate::keydb;
    use crate::pidgen::v3::UnsignedProductKey;

    #[test]
    fn pack_test_bink1998() {
        let key = ProductKey {
            inner: UnsignedProductKey {
                version: KeyVersion::Bink1998,
                upgrade: false,
                channel_id: 640,
                sequence_or_authinfo: 10550,
            },
            hash: 39185432,
            signature: 6939952665262054,
        };

        assert_eq!(key.to_string(), "D9924-R6BG2-39J83-RYKHF-W47TT");
    }

    #[test]
    fn verify_test_bink1998() {
        let keys = keydb::load_default_keys().unwrap();
        let ckp = keys.get_bink_by_id(0x2E).unwrap();
        let b24 = Base24::with_alphabet(Base24::ALPHABET_MS).unwrap();

        assert!(ProductKey::from_key(
            "D9924-R6BG2-39J83-RYKHF-W47TT",
            ckp,
            &b24,
            KeyVersion::Bink1998
        )
        .is_ok());
        assert!(ProductKey::from_key(
            "11111-R6BG2-39J83-RYKHF-W47TT",
            ckp,
            &b24,
            KeyVersion::Bink1998
        )
        .is_err());
    }

    #[test]
    fn verify_test_bink2002() {
        let keys = keydb::load_default_keys().unwrap();
        let ckp = keys.get_bink_by_id(0x54).unwrap();
        let b24 = Base24::with_alphabet(Base24::ALPHABET_MS).unwrap();

        assert!(ProductKey::from_key(
            "R882X-YRGC8-4KYTG-C3FCC-JCFDY",
            ckp,
            &b24,
            KeyVersion::Bink2002
        )
        .is_ok());
        assert!(ProductKey::from_key(
            "11111-YRGC8-4KYTG-C3FCC-JCFDY",
            ckp,
            &b24,
            KeyVersion::Bink2002
        )
        .is_err());
    }
}
