use anyhow::{Context, Result, anyhow};

use crate::App;
use crate::pidgen;
use crate::pidgen::v3::{KeyVersion, UnsignedProductKey};

pub fn execute(app: &App, version: KeyVersion, bink_id: u32, channel_id: u32, upgrade: bool) -> Result<()> {
    let mut rng = rand::rng();

	let ckp = app.keys.get_bink_by_id(bink_id).ok_or(anyhow!("could not find key with specified BINK ID"))?;

	let unsigned_key = UnsignedProductKey::new(version, channel_id, upgrade, &mut rng);

    let key = pidgen::v3::ProductKey::generate(unsigned_key, &mut rng, ckp).context("unable to generate key")?;

    eprintln!("PIDGEN3 {} key for BINK ID 0x{:02X}, channel ID {}:", version, bink_id, channel_id);
	println!("{}", key);

    Ok(())
}
