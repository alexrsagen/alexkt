// - Algorithm originally provided to the UMSKT project by "diamondggg"
//   "diamondggg" claims they are the originator of the code and that it was
//   created in tandem with an acquaintance who knows number theory.
//
// - Code ported from C/C++ to Rust by Alex Page
//   https://github.com/anpage/umskt-rs/blob/main/umskt/src/confid/black_box.rs
//
// - Rust code modified by Alexander Sagen <alexander@sagen.me>
//   Cleaned up, restructured slightly to fit this project

pub mod error;
use error::ConfirmationIdError;

mod installation_id;
pub use installation_id::InstallationId;

mod activation_mode;
pub use activation_mode::ActivationMode;

pub(super) mod math;

use std::mem::swap;

pub(super) const MOD: u64 = 0x16A6B036D7F2A79;
pub(super) const BAD: u64 = 0xffffffffffffffff;

pub(super) static F: [u64; 6] = [
    0,
    0x21840136c85381,
    0x44197b83892ad0,
    0x1400606322b3b04,
    0x1400606322b3b04,
    1,
];

#[derive(Copy, Clone)]
pub(super) struct TDivisor {
    u: [u64; 2],
    v: [u64; 2],
}

pub fn generate(id: &InstallationId) -> Result<String, ConfirmationIdError> {
    let mut keybuf: [u8; 16] = [0; 16];
    keybuf[..8].copy_from_slice(&id.hardware_id.to_le_bytes()[..8]);
    let product_id_mixed = id.product_id.mix();
    keybuf[8..16].copy_from_slice(&product_id_mixed.to_le_bytes()[..8]);
    let mut d_0: TDivisor = TDivisor {
        u: [0; 2],
        v: [0; 2],
    };

    let mut attempt = 0u8;
    while attempt as i32 <= 0x80_i32 {
        let mut u: [u8; 14] = [0; 14];
        u[7usize] = attempt;
        math::mix(&mut u, &keybuf, id.mode);
        let u_lo = u64::from_le_bytes(u[0..8].try_into().unwrap());
        let u_hi = u64::from_le_bytes(
            u[8..14]
                .iter()
                .chain([0, 0].iter())
                .cloned()
                .collect::<Vec<u8>>()[..]
                .try_into()
                .unwrap(),
        );
        let mut x2: u64 = math::ui128_quotient_mod(u_lo, u_hi);
        let x1: u64 = u_lo.wrapping_sub(x2.wrapping_mul(MOD));
        x2 = x2.wrapping_add(1);
        d_0.u[0usize] = math::residue_sub(
            math::residue_mul(x1, x1),
            math::residue_mul(43u64, math::residue_mul(x2, x2)),
        );
        d_0.u[1usize] = math::residue_add(x1, x1);
        if math::find_divisor_v(&mut d_0) != 0 {
            break;
        }
        attempt = attempt.wrapping_add(1);
    }

    if attempt as i32 > 0x80_i32 {
        return Err(ConfirmationIdError::Unlucky);
    }

    math::divisor_mul128(
        &(d_0.clone()),
        0x4e21b9d10f127c1_i64 as u64,
        0x40da7c36d44c_i64 as u64,
        &mut d_0,
    );

    let mut encoded_lo;
    let mut encoded_hi = 0;
    if d_0.u[0usize] == BAD {
        // we can not get the zero divisor, actually...
        encoded_lo = math::umul128(MOD.wrapping_add(2u64), MOD, &mut encoded_hi);
    } else if d_0.u[1usize] == BAD {
        encoded_lo = math::umul128(
            MOD.wrapping_add(1u64),
            d_0.u[0usize],
            &mut encoded_hi,
        );
        encoded_lo = encoded_lo.wrapping_add(MOD);
        encoded_hi = encoded_hi
            .wrapping_add((encoded_lo < MOD) as i32 as u64);
    } else {
        let x1_0: u64 = (if d_0.u[1usize] as i32 % 2_i32 != 0 {
            d_0.u[1usize].wrapping_add(MOD)
        } else {
            d_0.u[1usize]
        })
        .wrapping_div(2u64);
        let x2sqr: u64 = math::residue_sub(math::residue_mul(x1_0, x1_0), d_0.u[0usize]);
        let mut x2_0: u64 = math::residue_sqrt(x2sqr);
        if x2_0 == BAD {
            x2_0 = math::residue_sqrt(math::residue_mul(x2sqr, math::residue_inv(43u64)));
            encoded_lo = math::umul128(
                MOD.wrapping_add(1u64),
                MOD.wrapping_add(x2_0),
                &mut encoded_hi,
            );
            encoded_lo = encoded_lo.wrapping_add(x1_0);
            encoded_hi = encoded_hi
                .wrapping_add((encoded_lo < x1_0) as i32 as u64);
        } else {
            // points (-x1+x2, v(-x1+x2)) and (-x1-x2, v(-x1-x2))
            let mut x1a: u64 = math::residue_sub(x1_0, x2_0);
            let y1: u64 = math::residue_sub(
                d_0.v[0usize],
                math::residue_mul(d_0.v[1usize], x1a),
            );
            let mut x2a: u64 = math::residue_add(x1_0, x2_0);
            let y2: u64 = math::residue_sub(
                d_0.v[0usize],
                math::residue_mul(d_0.v[1usize], x2a),
            );
            if x1a > x2a {
                swap(&mut x1a, &mut x2a);
            }
            if (y1 ^ y2) & 1u64 != 0 {
                swap(&mut x1a, &mut x2a);
            }
            encoded_lo = math::umul128(MOD.wrapping_add(1u64), x1a, &mut encoded_hi);
            encoded_lo = encoded_lo.wrapping_add(x2a);
            encoded_hi = encoded_hi
                .wrapping_add((encoded_lo < x2a) as i32 as u64);
        }
    }

    let mut e_2 = [
        u32::from_le_bytes(encoded_lo.to_le_bytes()[0..4].try_into().unwrap()),
        u32::from_le_bytes(encoded_lo.to_le_bytes()[4..].try_into().unwrap()),
        u32::from_le_bytes(encoded_hi.to_le_bytes()[0..4].try_into().unwrap()),
        u32::from_le_bytes(encoded_hi.to_le_bytes()[4..].try_into().unwrap()),
    ];

    let mut decimal: [u8; 35] = [0; 35];
    let mut i = 0usize;
    while i < 35 {
        let c: u32 = (e_2[3usize]).wrapping_rem(10u32);
        e_2[3usize] = e_2[3usize].wrapping_div(10u32);
        let c2: u32 =
            ((c as u64) << 32_i32 | e_2[2usize] as u64).wrapping_rem(10u64) as u32;
        e_2[2usize] =
            ((c as u64) << 32_i32 | e_2[2usize] as u64).wrapping_div(10u64) as u32;
        let c3: u32 =
            ((c2 as u64) << 32_i32 | e_2[1usize] as u64).wrapping_rem(10u64) as u32;
        e_2[1usize] =
            ((c2 as u64) << 32_i32 | e_2[1usize] as u64).wrapping_div(10u64) as u32;
        let c4: u32 =
            ((c3 as u64) << 32_i32 | e_2[0usize] as u64).wrapping_rem(10u64) as u32;
        e_2[0usize] =
            ((c3 as u64) << 32_i32 | e_2[0usize] as u64).wrapping_div(10u64) as u32;
        decimal[34_usize.wrapping_sub(i)] = c4 as u8;
        i = i.wrapping_add(1);
    }

    let mut q = vec![0u8; 48];
    let mut i: usize = 0;
    let mut q_i = 0;
    while i < 7 {
        if i != 0 {
            q[q_i] = b'-';
            q_i += 1;
        }
        let p_0: &mut [u8] = &mut decimal[i.wrapping_mul(5)..];
        q[q_i] = (p_0[0] as i32 + '0' as i32) as u8;
        q[q_i + 1] = (p_0[1] as i32 + '0' as i32) as u8;
        q[q_i + 2] = (p_0[2] as i32 + '0' as i32) as u8;
        q[q_i + 3] = (p_0[3] as i32 + '0' as i32) as u8;
        q[q_i + 4] = (p_0[4] as i32 + '0' as i32) as u8;
        q[q_i + 5] = ((p_0[0] as i32
            + p_0[1] as i32 * 2_i32
            + p_0[2] as i32
            + p_0[3] as i32 * 2_i32
            + p_0[4] as i32)
            % 7_i32
            + '0' as i32) as u8;
        q_i = q_i.wrapping_add(6);
        i = i.wrapping_add(1);
    }

    let confirmation_id = String::from_utf8(q)?;
    Ok(confirmation_id)
}
