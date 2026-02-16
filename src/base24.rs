use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero};
use std::collections::{BTreeMap, VecDeque};

pub mod error;
use error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Base24 {
    alphabet_len: u32,
    encode_map: BTreeMap<u32, char>,
    decode_map: BTreeMap<char, u32>,
}

impl Base24 {
    pub const ALPHABET_KUON: &'static str = "ZAC2B3EF4GH5TK67P8RS9WXY";
    pub const ALPHABET_BASE24ORG: &'static str = "BCDFGHJKLMNPQRSTVWXZ6789";
    pub const ALPHABET_MS: &'static str = "BCDFGHJKMPQRTVWXY2346789";

    pub fn with_alphabet(alphabet: &str) -> Result<Base24> {
        if alphabet.len() > u32::MAX as usize {
            Err(Error::AlphabetLengthInvalid)
        } else {
            let alphabet_len = alphabet.len() as u32;

            let encode_map = alphabet
                .char_indices()
                .map(|(idx, chr)| (idx as u32, chr))
                .collect();

            let upper_decode_map: Vec<(char, u32)> = alphabet
                .to_uppercase()
                .char_indices()
                .map(|(idx, chr)| (chr, idx as u32))
                .collect();

            let lower_decode_map: Vec<(char, u32)> = alphabet
                .to_lowercase()
                .char_indices()
                .map(|(idx, chr)| (chr, idx as u32))
                .collect();

            let decode_map = upper_decode_map
                .into_iter()
                .chain(lower_decode_map.into_iter())
                .collect();

            Ok(Base24 {
                alphabet_len,
                encode_map,
                decode_map,
            })
        }
    }

    pub fn encode_bytes(&self, data: &[u8], include_padding: bool) -> Result<String> {
        let mut output = String::new();
        let chunks = data.len().div_ceil(4);
        for (i, chunk) in data.chunks(4).enumerate() {
            let mut value = u32::from_be_bytes([
                chunk.get(0).cloned().unwrap_or_default(),
                chunk.get(1).cloned().unwrap_or_default(),
                chunk.get(2).cloned().unwrap_or_default(),
                chunk.get(3).cloned().unwrap_or_default(),
            ]);

            let mut buf = VecDeque::with_capacity(7);
            for _ in 0..7 {
                let idx = value % self.alphabet_len;
                if idx > 0 || i < chunks - 1 || include_padding {
                    value = value.saturating_div(self.alphabet_len as u32);
                    buf.push_front(self.encode_map[&idx]);
                }
            }
            output.extend(buf);
        }
        Ok(output)
    }

    pub fn encode_biguint(&self, number: &BigUint) -> String {
        let mut z = number.clone();
        let mut out = VecDeque::new();

        for _ in 0..=24 {
            let (quo, rem) = z.div_rem(&BigUint::from(24_u32));
            z = quo;
            out.push_front(self.encode_map[&rem.to_u32().unwrap()]);
        }

        out.iter().collect()
    }

    fn decode_iter_to_u32<I: Iterator<Item = char>>(
        &self,
        iter: I,
        skip_invalid: bool,
    ) -> Result<Vec<u32>> {
        iter.into_iter()
            .filter_map(|c| {
                if skip_invalid {
                    self.decode_map.get(&c).map(|v| Ok(v.clone()))
                } else {
                    Some(
                        self.decode_map
                            .get(&c)
                            .cloned()
                            .ok_or(Error::DecodeUnsupportedCharacter(c)),
                    )
                }
            })
            .collect()
    }

    pub fn decode_iter_to_biguint<I: Iterator<Item = char>>(
        &self,
        iter: I,
        skip_invalid: bool,
    ) -> Result<BigUint> {
        Ok(self
            .decode_iter_to_u32(iter, skip_invalid)?
            .into_iter()
            .fold(BigUint::zero(), |mut acc, chunk| {
                acc *= 24_u32;
                acc += chunk;
                acc
            }))
    }

    pub fn decode_iter_to_bytes<I: Iterator<Item = char>>(
        &self,
        iter: I,
        skip_invalid: bool,
    ) -> Result<Vec<u8>> {
        self.decode_iter_to_u32(iter, skip_invalid)?
            .chunks(7)
            .map(|chunk| {
                chunk.into_iter().try_fold(0u32, |acc, idx| {
                    acc.checked_mul(self.alphabet_len)
                        .ok_or(Error::DecodeInvalidEncoding)?
                        .checked_add(*idx)
                        .ok_or(Error::DecodeInvalidEncoding)
                })
            })
            .try_fold(Vec::<u8>::new(), |mut acc, chunk| {
                acc.extend_from_slice(chunk?.to_be_bytes().as_slice());
                Ok(acc)
            })
    }

    pub fn decode_to_bytes(&self, data: &str, skip_invalid: bool) -> Result<Vec<u8>> {
        self.decode_iter_to_bytes(data.chars(), skip_invalid)
    }

    pub fn decode_to_biguint(&self, data: &str, skip_invalid: bool) -> Result<BigUint> {
        self.decode_iter_to_biguint(data.chars(), skip_invalid)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn decode_encode_equals_input() {
        use super::Base24;
        let input = "JTW3TJ7PFJ7V9CCMX84V9PFT8";
        let b24 = Base24::with_alphabet(Base24::ALPHABET_MS)
            .expect("unable to construct a Base24 decoder/encoder");
        let decoded = b24.decode_to_bytes(input, false).expect("unable to decode");
        let encoded = b24.encode_bytes(&decoded, false).expect("unable to encode");
        assert_eq!(input, encoded);
    }
}
