use anyhow::Result;

use crate::App;
use crate::pidgen;
use crate::pidgen::v2::KeyVariant;

pub fn execute(_app: &App, variant: KeyVariant) -> Result<()> {
    let mut rng = rand::rng();

    let key = pidgen::v2::Key::generate(&mut rng, variant);

    eprintln!("PIDGEN2 {} key:", variant);
	println!("{:?}", key);

    Ok(())
}
