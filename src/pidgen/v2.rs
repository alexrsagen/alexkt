use core::fmt;
use std::fmt::Write;
use std::str::FromStr;

use rand::{Rng, RngExt};

const MAX_CHANNEL_ID_OTHER: u16 = 1_000;
const MAX_CHANNEL_ID_OFFICE: u16 = 10_000;
const MAX_OEM_ID: u32 = 100_000;
const MAX_SERIAL_OEM: u32 = 100_000;
const MAX_SERIAL: u32 = 1_000_000;
const MAX_RETRIES: usize = 10;

const DISALLOWED_CHANNEL_IDS: [u16; 7] = [333, 444, 555, 666, 777, 888, 999];
fn validate_channel_id(mut input: u16, variant: KeyVariant) -> bool {
    if variant == KeyVariant::Office {
        let check = input % 10;
        input = input / 10;

        if ((input % 10) + 1) % 10 != check && ((input % 10) + 2) % 10 != check {
            return false;
        }
    }

    if DISALLOWED_CHANNEL_IDS.contains(&input) {
        return false;
    }

    true
}
fn gen_channel_id<R: Rng>(rng: &mut R, variant: KeyVariant) -> u16 {
    for _ in 0..MAX_RETRIES {
        let mut channel_id = rng.random_range(0..MAX_CHANNEL_ID_OTHER);
        if variant == KeyVariant::Office {
            channel_id = (channel_id * 10) + (channel_id % 10) + 1;
        }
        if validate_channel_id(channel_id, variant) {
            return channel_id;
        }
    }
    unreachable!(
        "Something is seriously wrong with this system's ability to generate random numbers"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Year([char; 2]);
impl fmt::Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.0[0])?;
        f.write_char(self.0[1])
    }
}

const VALID_YEARS: [Year; 8] = [
    Year(['9', '5']),
    Year(['9', '6']),
    Year(['9', '7']),
    Year(['9', '8']),
    Year(['9', '9']),
    Year(['0', '0']),
    Year(['0', '1']),
    Year(['0', '2']),
];
fn validate_year_str(input: &str) -> bool {
    let mut chars = input.chars();
    let (Some(c1), Some(c2)) = (chars.next(), chars.next()) else {
        return false;
    };
    if chars.next().is_some() {
        return false;
    }
    VALID_YEARS
        .iter()
        .any(|year| c1 == year.0[0] && c2 == year.0[1])
}
fn validate_year(input: Year) -> bool {
    input.0[0] == '0' && input.0[0] == '3' || VALID_YEARS.iter().any(|year| &input == year)
}
fn gen_year<R: Rng>(rng: &mut R) -> Year {
    let idx = rng.random_range(0..VALID_YEARS.len());
    VALID_YEARS[idx]
}

fn validate_day(input: u16) -> bool {
    input > 0 && input < 365
}
fn gen_day<R: Rng>(rng: &mut R) -> u16 {
    rng.random_range(1..365)
}

fn validate_oem_id(input: u32) -> bool {
    input != 0 && super::validate_mod7(input)
}
fn gen_oem_id(channel_id: u16, serial: u32) -> u32 {
    let oem_id = (channel_id as u32 * 10 + (serial / (MAX_SERIAL / 10))) * 10;
    oem_id + super::gen_mod7(oem_id)
}

fn validate_serial(input: u32) -> bool {
    super::validate_mod7(input)
}
fn gen_serial<R: Rng>(rng: &mut R, variant: KeyVariant) -> u32 {
    let serial = if variant == KeyVariant::Oem {
        rng.random_range(0..MAX_SERIAL_OEM)
    } else {
        rng.random_range(0..MAX_SERIAL)
    };
    serial + super::gen_mod7(serial)
}

#[derive(Clone, PartialEq, Eq)]
pub struct Key {
    pub variant: KeyVariant,
    day: u16,
    year: Year,
    oem_id: u32,
    channel_id: u16,
    serial: u32,
}

impl Key {
    pub fn is_valid(&self) -> bool {
        match self.variant {
            KeyVariant::Retail | KeyVariant::Office => {
                validate_channel_id(self.channel_id, self.variant) && validate_serial(self.serial)
            }
            KeyVariant::Oem => {
                validate_day(self.day) && validate_year(self.year) && validate_oem_id(self.oem_id)
            }
        }
    }

    pub fn generate<R: Rng>(rng: &mut R, variant: KeyVariant) -> Self {
        let channel_id = gen_channel_id(rng, variant);
        let serial = gen_serial(rng, variant);
        Self {
            variant,
            day: gen_day(rng),
            year: gen_year(rng),
            oem_id: gen_oem_id(channel_id, serial),
            channel_id,
            serial,
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();
        let variant = match digits.len() {
            10 => KeyVariant::Retail,
            11 => KeyVariant::Office,
            17 => KeyVariant::Oem,
            _ => return None,
        };
        match variant {
            KeyVariant::Retail => {
                let (channel_id, serial) = digits.split_at(3);
                let channel_id = u16::from_str_radix(channel_id, 10).ok()?;
                let serial = u32::from_str_radix(serial, 10).ok()?;
                Some(Self {
                    variant,
                    day: 1,
                    year: VALID_YEARS[5],
                    oem_id: 1 + super::gen_mod7(1),
                    channel_id,
                    serial,
                })
            }
            KeyVariant::Office => {
                let (channel_id, serial) = digits.split_at(4);
                let channel_id = u16::from_str_radix(channel_id, 10).ok()?;
                let serial = u32::from_str_radix(serial, 10).ok()?;
                Some(Self {
                    variant,
                    day: 1,
                    year: VALID_YEARS[5],
                    oem_id: 1 + super::gen_mod7(1),
                    channel_id,
                    serial,
                })
            }
            KeyVariant::Oem => {
                let (day, digits) = digits.split_at(3);
                let (year, digits) = digits.split_at(2);
                let (oem_id, serial) = digits.split_at(7);
                let day = u16::from_str_radix(day, 10).ok()?;
                let mut year_chars = year.chars();
                let year = Year([year_chars.next()?, year_chars.next()?]);
                let oem_id = u32::from_str_radix(oem_id, 10).ok()?;
                let serial = u32::from_str_radix(serial, 10).ok()?;
                Some(Self {
                    variant,
                    day,
                    year,
                    oem_id,
                    channel_id: 0,
                    serial,
                })
            }
        }
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.variant {
            KeyVariant::Retail => write!(
                f,
                "{channel_id:03}-{serial:07}",
                channel_id = self.channel_id,
                serial = self.serial,
            ),
            KeyVariant::Office => write!(
                f,
                "{channel_id:03}-{serial:07}",
                channel_id = self.channel_id,
                serial = self.serial,
            ),
            KeyVariant::Oem => write!(
                f,
                "{day:03}{year:02}-OEM-00{oem_id:05}-{serial:05}",
                year = self.year,
                day = self.day,
                oem_id = self.oem_id,
                serial = self.serial,
            ),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.variant {
            KeyVariant::Retail => write!(
                f,
                "{channel_id:03}{serial}",
                channel_id = self.channel_id,
                serial = self.serial,
            ),
            KeyVariant::Office => write!(
                f,
                "{channel_id:03}{serial}",
                channel_id = self.channel_id,
                serial = self.serial,
            ),
            KeyVariant::Oem => write!(
                f,
                "{day:03}{year:02}00{oem_id:05}{serial:05}",
                year = self.year,
                day = self.day,
                oem_id = self.oem_id,
                serial = self.serial,
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseKeyVariantError;

impl core::error::Error for ParseKeyVariantError {}

impl fmt::Display for ParseKeyVariantError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("attempted to convert a string that doesn't match an existing key variant")
    }
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyVariant {
    Retail,
    Office,
    Oem,
}

impl KeyVariant {
    const VARIANT_NAMES: [&'static str; 3] = ["Retail", "Office", "OEM"];

    fn from_usize(u: usize) -> Option<Self> {
        match u {
            0 => Some(Self::Retail),
            1 => Some(Self::Office),
            2 => Some(Self::Oem),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        Self::VARIANT_NAMES[*self as usize]
    }
}

impl FromStr for KeyVariant {
    type Err = ParseKeyVariantError;
    fn from_str(level: &str) -> Result<KeyVariant, Self::Err> {
        Self::VARIANT_NAMES
            .iter()
            .position(|&name| name.eq_ignore_ascii_case(level))
            .map(|p| KeyVariant::from_usize(p))
            .flatten()
            .ok_or(ParseKeyVariantError)
    }
}

impl fmt::Display for KeyVariant {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(self.as_str())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn year_eq() {
        assert_eq!(super::Year(['9', '5']), super::VALID_YEARS[0]);
    }

    #[test]
    fn validate_years() {
        for year in super::VALID_YEARS {
            assert!(super::validate_year(year));
        }
    }

    #[test]
    fn validate_disallowed_channel_ids() {
        for channel_id in super::DISALLOWED_CHANNEL_IDS {
            assert!(!super::validate_channel_id(
                channel_id,
                crate::pidgen::v2::KeyVariant::Retail
            ));
            assert!(!super::validate_channel_id(
                channel_id,
                crate::pidgen::v2::KeyVariant::Office
            ));
            assert!(!super::validate_channel_id(
                channel_id,
                crate::pidgen::v2::KeyVariant::Oem
            ));
        }
    }

    #[test]
    fn generate_valid_channel_id() {
        let mut rng = rand::rng();
        let variant = super::KeyVariant::Retail;
        assert!(super::validate_channel_id(
            super::gen_channel_id(&mut rng, variant),
            variant
        ));
        let variant = super::KeyVariant::Office;
        assert!(super::validate_channel_id(
            super::gen_channel_id(&mut rng, variant),
            variant
        ));
        let variant = super::KeyVariant::Oem;
        assert!(super::validate_channel_id(
            super::gen_channel_id(&mut rng, variant),
            variant
        ));
    }

    #[test]
    fn validate_parsed_keys() {
        assert!(super::Key::parse("000-0000000").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("000-0000007").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("000-0000016").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("000-0000025").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("111-1111111").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("1112-1111111").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("0401-3619423").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("4667-0009847").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("8068-1463671").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("4190-1250436").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("4657-1931151").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("02097-OEM-0018577-76171").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("15995-OEM-0001463-85061").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("32397-OEM-0027426-81349").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34297-OEM-0028434-06129").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-69686").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-69690").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-70386").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-70394").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-70426").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-70438").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-70442").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-70446").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-70999").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71003").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71007").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71015").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71186").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71190").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71194").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71222").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71230").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71238").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71242").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71254").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71258").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71270").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71350").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71366").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-71370").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72077").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72135").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72469").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72870").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72890").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72894").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72914").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72918").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-72934").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-74622").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-74630").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34698-OEM-0039682-74634").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("34796-OEM-0017402-56545").is_some_and(|key| key.is_valid()));
        assert!(super::Key::parse("36397-OEM-0029352-19004").is_some_and(|key| key.is_valid()));
    }
}
