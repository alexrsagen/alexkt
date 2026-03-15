pub mod error;
use error::ProductIdError;

use core::fmt;
use std::str::FromStr;

#[derive(Debug, Default, Clone, Copy)]
pub struct ProductId {
    pub id: [u32; 4],
    pub oem: bool,
}

impl ProductId {
    fn calculate_check_digit(pid: u32) -> u32 {
        let mut i = 0;
        let mut j = pid;
        while j > 0 {
            let k = j % 10;
            j /= 10;
            i += k;
        }
        ((10 * pid) - (i % 7)) + 7
    }

    pub fn parse(s: &str) -> Result<Self, ProductIdError> {
        let mut product_id = Self { id: [0u32; 4], oem: false };
        let mut buf = String::new();
        let mut total_count = 0usize;

        let mut chars_iter = s.chars().peekable();
        while let Some(c) = chars_iter.next() {
            if c.is_whitespace() || c == '-' {
                continue;
            }

            if c.is_ascii_digit() {
                buf.push(c);
            } else if c.is_ascii_alphabetic() {
                let upper = c.to_ascii_uppercase();
                if upper == 'O' || upper == 'E' || upper == 'M' {
                    buf.push(upper);
                }
            }

            total_count += 1;

            if total_count == 5 {
                product_id.id[0] = u32::from_str_radix(&buf, 10).map_err(|_e| ProductIdError::InvalidCharacter)?;
                buf.clear();
            } else if total_count == 8 {
                if buf == "OEM" {
                    product_id.oem = true;
                } else {
                    product_id.id[1] = u32::from_str_radix(&buf, 10).map_err(|_e| ProductIdError::InvalidCharacter)?;
                }
                buf.clear();
            } else if total_count == 15 {
                if product_id.oem {
                    product_id.id[1] = u32::from_str_radix(&buf[2..5], 10).map_err(|_e| ProductIdError::InvalidCharacter)?;
                    product_id.id[2] = buf.chars().nth(4).unwrap().to_digit(10).ok_or(ProductIdError::InvalidCharacter)? * 100000;
                    product_id.id[3] = u32::from_str_radix(&buf[0..2], 10).map_err(|_e| ProductIdError::InvalidCharacter)? * 1000;
                } else {
                    product_id.id[2] = u32::from_str_radix(&buf, 10).map_err(|_e| ProductIdError::InvalidCharacter)?;
                }
                buf.clear();
            } else if total_count == 20 {
                if product_id.oem {
                    product_id.id[2] += u32::from_str_radix(&buf, 10).map_err(|_e| ProductIdError::InvalidCharacter)?;
                    product_id.id[2] = Self::calculate_check_digit(product_id.id[2]);
                } else {
                    product_id.id[3] = u32::from_str_radix(&buf, 10).map_err(|_e| ProductIdError::InvalidCharacter)?;
                }
                buf.clear();
                break;
            }
        }

        if total_count < 20 {
            return Err(ProductIdError::TooShort);
        }

        Ok(product_id)
    }

    pub fn mix(&self) -> u64 {
        (self.id[0] as u64) << 41 |
        (self.id[1] as u64) << 58 |
        (self.id[2] as u64) << 17 |
        (self.id[3] as u64)
    }
}

impl FromStr for ProductId {
    type Err = ProductIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for ProductId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let group_0 = self.id[0];
        if self.oem {
            let group_2 = Self::calculate_check_digit(self.id[3] * 10 + self.id[1] * 10);
            let group_2_digit_count = group_2.checked_ilog10().unwrap_or(0) + 1;
            let group_2_5th_digit = (group_2 / 10_u32.pow(group_2_digit_count - 5)) % 10;
            let group_3 = self.id[2] / 10 - group_2_5th_digit * 100000;
            f.pad(format!("{group_0:05}-OEM-{group_2:07}-{group_3:05}").as_str())
        } else {
            let group_1 = self.id[1];
            let group_2 = self.id[2];
            let group_3 = self.id[3];
            f.pad(format!("{group_0:05}-{group_1:03}-{group_2:07}-{group_3:05}").as_str())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_and_format_1() {
        use super::ProductId;
        let input = "55724-012-9869442-22571";
        let product_id = ProductId::parse(input).expect("unable to parse product ID");
        let output = product_id.to_string();
        assert_eq!(input, output);
    }

    #[test]
    fn parse_and_format_2_oem() {
        use super::ProductId;
        let input = "55724-OEM-2211906-00106";
        let product_id = ProductId::parse(input).expect("unable to parse product ID");
        let output = product_id.to_string();
        assert_eq!(input, output);
    }
}
