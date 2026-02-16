use std::path::Path;

use anyhow::{Context, Result};

use crate::crypto::Point;
use crate::{bink, keydb, App};

pub fn execute(app: &App, file_path: Option<&Path>) -> Result<()> {
	let binkeys = if let Some(file_path) = file_path {
		bink::get_binkey_resources_from_file(file_path).context("Unable to get BINK info from specified file")?
	} else {
		bink::get_binkey_resources_from_system_file().context("Unable to get BINK info from current system")?
	};

	for binkey in &binkeys {
		if binkey.header.authlen.is_some() {
			println!("BINK2002 header:");
			// println!("Operating System:  Windows Server 2003 / XP SP2 x64");
		} else {
			println!("BINK1998 header:");
			// println!("Operating System:  Windows 98 / XP x86");
		};

		println!("Identifier:        0x{:04X}", binkey.id);
		for product in app.keys.get_products_by_bink_id(binkey.id) {
			match product {
				keydb::ProductOrFlavourRef::Product { product } => {
					println!("Possible product:  {}", product.name);
				}
				keydb::ProductOrFlavourRef::ProductFlavour { product, flavour } => {
					println!("Possible product:  {} {}", product.name, flavour.name);
				}
			}
		}
		println!("sizeof(BINKEY):    {}", binkey.len());
		println!("Header Length:     {}", binkey.header.header_words);
		println!(
			"Checksum:          0x{:08X} ({})",
			binkey.header.checksum, binkey.header.checksum
		);
		if let Some(version_date) = binkey.header.version_date() {
			println!("Creation Date:     {}", version_date);
		}
		println!(
			"ECC Key Size:      {} bits ({} DWORDs)",
			binkey.header.key_bits(),
			binkey.header.key_words
		);
		println!("Hash Length:       {} bits", binkey.header.hash_len);
		println!("Signature Length:  {} bits", binkey.header.sig_len);
		if let Some(authlen) = &binkey.header.authlen {
			println!("Auth Field Length: {} bits", authlen);
		}
		if let Some(pidlen) = &binkey.header.pidlen {
			println!("Product ID Length: {} bits", pidlen);
		}
		println!();
		println!("BINK Elliptic Curve Parameters:");
		println!("Finite Field Order p:");
		println!("Hex: 0x{:X}", binkey.curve.p);
		println!("Dec: {}", binkey.curve.p);
		println!();
		println!("Curve Parameter a:");
		println!("Hex: 0x{:X}", binkey.curve.a);
		println!("Dec: {}", binkey.curve.a);
		println!();
		println!("Curve Parameter b:");
		println!("Hex: 0x{:X}", binkey.curve.b);
		println!("Dec: {}", binkey.curve.b);
		println!();
		if let Point::Point { x, y } = &binkey.public.g {
			println!("Base Point x-coordinate Gx:");
			println!("Hex: 0x{:X}", x);
			println!("Dec: {}", x);
			println!();
			println!("Base Point y-coordinate Gy:");
			println!("Hex: 0x{:X}", y);
			println!("Dec: {}", y);
			println!();
		}
		if let Point::Point { x, y } = &binkey.public.k {
			println!("Public Key x-coordinate Kx:");
			println!("Hex: 0x{:X}", x);
			println!("Dec: {}", x);
			println!();
			println!("Public Key y-coordinate Ky:");
			println!("Hex: 0x{:X}", y);
			println!("Dec: {}", y);
			println!();

			let ky_neg = &binkey.curve.p - y;

			println!("Negative of Public Key y-coordinate Ky (-Ky):");
			println!("Hex: 0x{:X}", ky_neg);
			println!("Dec: {}", ky_neg);
			println!();
		}

		// load hardcoded pre-factored keys
		// TODO: implement ECDLP factorization (using Pollard's Rho / Kangaroo algorithm, maybe in CUDA?)
		if let Some(bink) = app.keys.get_bink_by_id(binkey.id) {
			println!("Order of base point G, n:");
			let order = &bink.private.n;
			println!("Hex: 0x{:X}", order);
			println!("Dec: {}", order);
			println!();

			println!("Private Key:");
			println!("Hex: 0x{:X}", &bink.private.key);
			println!("Dec: {}", &bink.private.key);
			println!();
		}
	}

	Ok(())
}