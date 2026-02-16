pub mod v2;
pub mod v3;
pub mod error;

pub(self) fn gen_mod7(input: u32) -> u32 {
	let mut sum = 0;
	let mut check = input.clone();
	while check != 0 {
		sum += check % 10;
		check /= 10;
	}
	7 - (sum % 7)
}

pub(self) fn validate_mod7(input: u32) -> bool {
	let mut sum = 0;
	let mut check = input.clone();
	while check != 0 {
		sum += check % 10;
		check /= 10;
	}
	sum % 7 == 0
}
