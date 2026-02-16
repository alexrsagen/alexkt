use core::f64::consts::{LN_10, LN_2};
use core::fmt;

/// size in bytes of 1 byte
pub const B: u64 = 1;

/// size in bytes of 1 kilobyte
pub const KB: u64 = UnitBase10::THOUSAND;
/// size in bytes of 1 megabyte
pub const MB: u64 = UnitBase10::MILLION;
/// size in bytes of 1 gigabyte
pub const GB: u64 = UnitBase10::MILLIARD;
/// size in bytes of 1 terabyte
pub const TB: u64 = UnitBase10::BILLION;
/// size in bytes of 1 petabyte
pub const PB: u64 = UnitBase10::BILLIARD;
/// size in bytes of 1 exabyte
pub const EB: u64 = UnitBase10::TRILLION;
/// size in bytes of 1 zettabyte
pub const ZB: u128 = UnitBase10::TRILLIARD;
/// size in bytes of 1 yottabyte
pub const YB: u128 = UnitBase10::QUADRILLION;
/// size in bytes of 1 ronnabyte
pub const RB: u128 = UnitBase10::QUADRILLIARD;
/// size in bytes of 1 quettabyte
pub const QB: u128 = UnitBase10::QUINTILLION;

/// size in bytes of 1 kibibyte
pub const KIB: u64 = UnitBase2::THOUSAND;
/// size in bytes of 1 mebibyte
pub const MIB: u64 = UnitBase2::MILLION;
/// size in bytes of 1 gibibyte
pub const GIB: u64 = UnitBase2::MILLIARD;
/// size in bytes of 1 tebibyte
pub const TIB: u64 = UnitBase2::BILLION;
/// size in bytes of 1 pebibyte
pub const PIB: u64 = UnitBase2::BILLIARD;
/// size in bytes of 1 exbibyte
pub const EIB: u64 = UnitBase2::TRILLION;
/// size in bytes of 1 zebibyte
pub const ZIB: u128 = UnitBase2::TRILLIARD;
/// size in bytes of 1 yobibyte
pub const YIB: u128 = UnitBase2::QUADRILLION;
/// size in bytes of 1 robibyte
pub const RIB: u128 = UnitBase2::QUADRILLIARD;
/// size in bytes of 1 quebibyte
pub const QIB: u128 = UnitBase2::QUINTILLION;

pub type BytesBase2 = Bytes<UnitBase2>;
pub type BytesBase10 = Bytes<UnitBase10>;

pub fn from_bytes_base2(bytes: u64) -> BytesBase2 {
	BytesBase2::from_bytes(bytes as f64)
}

pub fn from_bytes_base10(bytes: u64) -> BytesBase10 {
	BytesBase10::from_bytes(bytes as f64)
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Bytes<U> {
	prefix: Prefix,
	value: f64,
	unit: U,
}

impl<U: Unit> Bytes<U> {
	pub fn from_prefix_value(prefix: Prefix, value: f64) -> Self {
		Self { prefix, value, unit: U::new() }
	}

	pub fn from_bytes(bytes: f64) -> Self {
		let prefix = U::max_prefix_for_value(bytes);
		Self::from_prefix_value(prefix, bytes / U::thousand_value_f64().powi(prefix.exponent() as i32))
	}

	pub fn to_bytes(&self) -> f64 {
		(self.value * U::prefix_value_f64(self.prefix)).ceil()
	}

	pub fn to_max_prefix(&self) -> Self {
		Self::from_bytes(self.to_bytes())
	}
}

impl<U: Unit> fmt::Display for Bytes<U> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use fmt::{Alignment, Write};

		let symbol = U::prefix_symbol(self.prefix);
		let unit = if symbol.is_empty() {
			'b'
		} else {
			'B'
		};

		// avoid using .trunc(), undefined symbol '_trunk' in VC8 CRT
		let value_floor = self.value.floor();
		let value_fract = self.value - value_floor;

		// count expected amount of chars in output representation of value
		let digit_chars_count = digit_count(value_floor);
		let decimal_chars_count = if value_fract == 0.0 {
			0
		} else {
			3 // decimal point + 2 digits
		};

		let symbol_chars_count = symbol.chars().count();

		// count expected amount of chars in output
		// (digits [+ decimal point + 2 digits] + space + symbol + unit)
		let chars_count = digit_chars_count + decimal_chars_count + 1 + symbol_chars_count + 1;

		// write left padding, for right alignment
		if let (Some(width), Some(Alignment::Right)) = (f.width(), f.align()) {
			for _ in 0..width.saturating_sub(chars_count) {
				f.write_char(f.fill())?;
			}
		}

		// write value (digits [+ decimal point + 2 digits])
		if value_fract == 0.0 {
			write!(f, "{:.0}", self.value)?;
		} else {
			write!(f, "{:.2}", self.value)?;
		}

		// write space
		f.write_char(f.fill())?;

		// write center padding, for center alignment
		if let (Some(width), Some(Alignment::Center)) = (f.width(), f.align()) {
			for _ in 0..width.saturating_sub(chars_count) {
				f.write_char(f.fill())?;
			}
		}

		// write symbol and unit
		f.write_str(symbol)?;
		f.write_char(unit)?;

		// write right padding, for left alignment
		if let (Some(width), Some(Alignment::Left)) = (f.width(), f.align()) {
			for _ in 0..width.saturating_sub(chars_count) {
				f.write_char(f.fill())?;
			}
		}

		Ok(())
	}
}


#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Prefix {
	/// One
	One,
	/// One thousand: kilo/kibi
	Thousand,
	/// One million: mega/mebi
	Million,
	/// One milliard: giga/gibi
	Milliard,
	/// One billion: tera/tebi
	Billion,
	/// One billiard: peta/pebi
	Billiard,
	/// One trillion: exa/exbi
	Trillion,
	/// One trilliard: zetta/zebi
	Trilliard,
	/// One quadrillion: yotta/yobi
	Quadrillion,
	/// One quadrilliard: ronna/robi
	Quadrilliard,
	/// One quintillion: quetta/quebi
	Quintillion,
}

impl Prefix {
	const ALL: [Self; 11] = [
		Self::One,
		Self::Thousand,
		Self::Million,
		Self::Milliard,
		Self::Billion,
		Self::Billiard,
		Self::Trillion,
		Self::Trilliard,
		Self::Quadrillion,
		Self::Quadrilliard,
		Self::Quintillion,
	];

	fn exponent(&self) -> u32 {
		match self {
			Self::One => 0,
			Self::Thousand => 1,
			Self::Million => 2,
			Self::Milliard => 3,
			Self::Billion => 4,
			Self::Billiard => 5,
			Self::Trillion => 6,
			Self::Trilliard => 7,
			Self::Quadrillion => 8,
			Self::Quadrilliard => 9,
			Self::Quintillion => 10,
		}
	}

	fn max(exp: u32) -> Self {
		// short-circuit
		if exp < Self::One.exponent() {
			return Self::One;
		}

		for e in Self::ALL.iter().skip(1).rev() {
			if exp >= e.exponent() {
				return *e;
			}
		}

		return Self::ALL[0];
	}
}

impl Ord for Prefix {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.exponent().cmp(&other.exponent())
	}
}

impl PartialOrd for Prefix {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.exponent().partial_cmp(&other.exponent())
	}
}

pub trait Unit {
	fn new() -> Self;
	fn thousand_value() -> u64;
	fn thousand_value_f64() -> f64 {
		Self::thousand_value() as f64
	}
	fn thousand_value_ln() -> f64 {
		Self::thousand_value_f64().ln()
	}
	fn max_prefix_for_value(value: f64) -> Prefix {
		Prefix::max((value.ln() / Self::thousand_value_ln()).floor() as u32)
	}
	fn prefix_value(prefix: Prefix) -> u64 {
		Self::thousand_value().pow(prefix.exponent())
	}
	fn prefix_value_f64(prefix: Prefix) -> f64 {
		Self::thousand_value_f64().powi(prefix.exponent() as i32)
	}
	fn prefix_name(prefix: Prefix) -> &'static str;
	fn prefix_symbol(prefix: Prefix) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitBase2;

impl UnitBase2 {
	pub const LN_THOUSAND: f64 = LN_2 * 10.0;
	pub const THOUSAND: u64 = 2u64.pow(10);
	pub const MILLION: u64 = 2u64.pow(20);
	pub const MILLIARD: u64 = 2u64.pow(30);
	pub const BILLION: u64 = 2u64.pow(40);
	pub const BILLIARD: u64 = 2u64.pow(50);
	pub const TRILLION: u64 = 2u64.pow(60);
	pub const TRILLIARD: u128 = 2u128.pow(70);
	pub const QUADRILLION: u128 = 2u128.pow(80);
	pub const QUADRILLIARD: u128 = 2u128.pow(90);
	pub const QUINTILLION: u128 = 2u128.pow(100);
}

impl Unit for UnitBase2 {
	fn new() -> Self {
		UnitBase2
	}

	fn thousand_value() -> u64 {
		Self::THOUSAND
	}

	fn thousand_value_ln() -> f64 {
		Self::LN_THOUSAND
	}

	fn prefix_name(prefix: Prefix) -> &'static str {
		match prefix {
			Prefix::One => "",
			Prefix::Thousand => "kibi",
			Prefix::Million => "mebi",
			Prefix::Milliard => "gibi",
			Prefix::Billion => "tebi",
			Prefix::Billiard => "pebi",
			Prefix::Trillion => "exbi",
			Prefix::Trilliard => "zebi",
			Prefix::Quadrillion => "yobi",
			Prefix::Quadrilliard => "robi",
			Prefix::Quintillion => "quebi",
		}
	}

	fn prefix_symbol(prefix: Prefix) -> &'static str {
		match prefix {
			Prefix::One => "",
			Prefix::Thousand => "Ki",
			Prefix::Million => "Mi",
			Prefix::Milliard => "Gi",
			Prefix::Billion => "Ti",
			Prefix::Billiard => "Pi",
			Prefix::Trillion => "Ei",
			Prefix::Trilliard => "Zi",
			Prefix::Quadrillion => "Yi",
			Prefix::Quadrilliard => "Ri",
			Prefix::Quintillion => "Qi",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitBase10;

#[allow(unused)]
impl UnitBase10 {
	pub const LN_THOUSAND: f64 = LN_10 * 3.0;
	pub const THOUSAND: u64 = 1_000u64.pow(1);
	pub const MILLION: u64 = 1_000u64.pow(2);
	pub const MILLIARD: u64 = 1_000u64.pow(3);
	pub const BILLION: u64 = 1_000u64.pow(4);
	pub const BILLIARD: u64 = 1_000u64.pow(5);
	pub const TRILLION: u64 = 1_000u64.pow(6);
	pub const TRILLIARD: u128 = 1_000u128.pow(7);
	pub const QUADRILLION: u128 = 1_000u128.pow(8);
	pub const QUADRILLIARD: u128 = 1_000u128.pow(9);
	pub const QUINTILLION: u128 = 1_000u128.pow(10);
}

impl Unit for UnitBase10 {
	fn new() -> Self {
		UnitBase10
	}

	fn thousand_value() -> u64 {
		Self::THOUSAND
	}

	fn thousand_value_ln() -> f64 {
		Self::LN_THOUSAND
	}

	fn prefix_name(prefix: Prefix) -> &'static str {
		match prefix {
			Prefix::One => "",
			Prefix::Thousand => "kilo",
			Prefix::Million => "mega",
			Prefix::Milliard => "giga",
			Prefix::Billion => "tera",
			Prefix::Billiard => "peta",
			Prefix::Trillion => "exa",
			Prefix::Trilliard => "zetta",
			Prefix::Quadrillion => "yotta",
			Prefix::Quadrilliard => "ronna",
			Prefix::Quintillion => "quetta",
		}
	}

	fn prefix_symbol(prefix: Prefix) -> &'static str {
		match prefix {
			Prefix::One => "",
			Prefix::Thousand => "k",
			Prefix::Million => "M",
			Prefix::Milliard => "G",
			Prefix::Billion => "T",
			Prefix::Billiard => "P",
			Prefix::Trillion => "E",
			Prefix::Trilliard => "Z",
			Prefix::Quadrillion => "Y",
			Prefix::Quadrilliard => "R",
			Prefix::Quintillion => "Q",
		}
	}
}

/// expects input to be truncated
fn digit_count(n: f64) -> usize {
	let n = n as u64;
	let base = 10;
    let mut power = base;
    let mut count = 1;
    while n >= power {
        count += 1;
        if let Some(new_power) = power.checked_mul(base) {
            power = new_power;
        } else {
            break;
        }
    }
    count
}