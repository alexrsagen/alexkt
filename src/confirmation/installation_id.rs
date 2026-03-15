use crate::product_id::ProductId;

use super::ActivationMode;
use super::error::InstallationIdError;
use super::math;

#[derive(Debug, Default, Clone, Copy)]
pub struct InstallationId {
    pub hardware_id: u64,
    pub product_id: ProductId,
    // pub key_sha1: u16,
    pub version: u8,
    pub mode: ActivationMode,
}

impl InstallationId {
    pub fn parse(s: &str, mode: ActivationMode, product_id: Option<ProductId>) -> Result<Self, InstallationIdError> {
        let mut bytes = [0u8; 19];
        let mut bytes_len = 0usize;

        let mut count = 0usize;
        let mut total_count = 0usize;
        let mut check = 0;

        let mut chars_iter = s.chars().peekable();
        while let Some(c) = chars_iter.next() {
            if c.is_whitespace() || c == '-' {
                continue;
            }

            let d = c.to_digit(10).ok_or(InstallationIdError::InvalidCharacter)? as u8;

            if count == 5 || chars_iter.peek().is_none() {
                if total_count >= 45 && count < 5 {
                    return Err(InstallationIdError::TooLong);
                } else if total_count != 41 && count < 5 {
                    return Err(InstallationIdError::TooShort);
                } else if d != check % 7 {
                    return Err(InstallationIdError::InvalidCheckDigit);
                }

                check = 0;
                count = 0;
                continue;
            }

            if count.wrapping_rem(2) == 0 {
                check = check.wrapping_add(d);
            } else {
                check = check.wrapping_add(d * 2);
            }

            count += 1;
            total_count += 1;
            if total_count > 45 {
                return Err(InstallationIdError::TooLong);
            }

            let mut carry = d;
            for i in 0..bytes_len {
                let x = bytes[i] as u32 * 10 + carry as u32;
                bytes[i] = (x & 0xFF) as u8;
                carry = (x >> 8) as u8;
            }

            if carry != 0 {
                assert!(bytes_len < bytes.len());
                bytes[bytes_len] = carry;
                bytes_len += 1;
            }
        }

        if total_count != 41 && total_count < 45 {
            return Err(InstallationIdError::TooShort);
        }

		if total_count == 41 {
			math::unmix(&mut bytes[0..17], &mode.installation_id_key(), mode);
		} else {
			math::unmix(&mut bytes, &mode.installation_id_key(), mode);
		}

        if bytes[18] >= 0x10 {
            return Err(InstallationIdError::UnknownVersion);
        }

        if mode.is_office() {
            let buffer: [u32; 5] = [
                u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
                u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
                u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
                u32::from_le_bytes([bytes[16], bytes[17], bytes[18], 0]),
            ];

            let v1 = (buffer[3] & 0xFFFFFFF8) | 2;
            let v2 = ((buffer[3] & 7) << 29) | (buffer[2] >> 3);
            let hardware_id = ((v1 as u64) << 32) | v2 as u64;

            let version = (buffer[0] & 7) as u8;

            let product_id = product_id.ok_or(InstallationIdError::UnspecifiedProductID)?;

            Ok(InstallationId { hardware_id, product_id, version, mode })
        } else {
            let hardware_id_bytes: [u8; 8] = bytes[0..8].try_into().unwrap();
            let hardware_id = u64::from_le_bytes(hardware_id_bytes);

            let product_id_low_bytes: [u8; 8] = bytes[8..16].try_into().unwrap();
            let product_id_low = u64::from_le_bytes(product_id_low_bytes);
            let product_id_high = bytes[16] as u64;

            let product_id = ProductId {
                id: [
                    (product_id_low & ((1 << 17) - 1)) as u32,
                    ((product_id_low >> 17) & ((1 << 10) - 1)) as u32,
                    ((product_id_low >> 27) & ((1 << 24) - 1)) as u32,
                    ((product_id_low >> 55) | (product_id_high << 9)) as u32,
                ],
                oem: false,
            };

            // let key_sha1_bytes: [u8; 2] = bytes[17..19].try_into().unwrap();
            // let key_sha1 = u16::from_le_bytes(key_sha1_bytes);

            let version = ((product_id_low >> 52) & 7) as u8;
			if total_count == 41 && version != 4 {
				return Err(InstallationIdError::UnknownVersion);
			} else if total_count != 41 && version != 5 {
				return Err(InstallationIdError::UnknownVersion);
			}

            Ok(InstallationId { hardware_id, product_id, version, mode })
        }
    }
}
