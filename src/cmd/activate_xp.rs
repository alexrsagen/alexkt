use anyhow::{Context, Result};
use log::info;

use crate::{App, confirmation};
use crate::confirmation::{ActivationMode, InstallationId};
use crate::license_agent::LicenseAgent;
use crate::product_id::ProductId;

pub fn execute(_app: &App, force: bool) -> Result<()> {
	// initialize license agent
	let mut lic = LicenseAgent::new().context("Unable to initialize license agent")?;

	// check if system is already activated
	let expiration_info = lic.get_expiration_info().context("Unable to get license expiration info")?;
	if expiration_info.is_activated() && !force {
		println!("License manager says that the system is already activated");
		return Ok(());
	}

	// get and parse product ID
	let product_id_str = lic.get_product_id().context("Unable to get product ID")?;
	let product_id = ProductId::parse(&product_id_str).context("Unable to parse product ID")?;
	info!("Product ID:      {}", product_id);

	// generate and parse installation ID
	let installation_id_str = lic.generate_installation_id().context("Unable to generate installation ID")?;
	info!("Installation ID: {}", installation_id_str);
	let installation_id = InstallationId::parse(&installation_id_str, ActivationMode::Windows, Some(product_id)).context("Unable to parse installation ID")?;

	// generate and deposit confirmation ID
	let confirmation_id = confirmation::generate(&installation_id).context("Unable to generate confirmation ID")?;
	lic.deposit_confirmation_id(&confirmation_id).context("Unable to deposit confirmation ID")?;
	info!("Confirmation ID: {}", confirmation_id);

	println!("Successful activation");

	Ok(())
}
