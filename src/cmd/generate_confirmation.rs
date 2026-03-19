use anyhow::Context;

use anyhow::Result;
use crate::App;
use crate::product_id::ProductId;
use crate::confirmation::{ActivationMode, InstallationId, generate};

pub fn execute(_app: &App, installation_id: &str, mode: ActivationMode, product_id: Option<ProductId>) -> Result<()> {
    let installation_id = InstallationId::parse(installation_id, mode, product_id).context("unable to parse installation ID")?;
    let confirmation_id = generate(&installation_id).context("unable to generate confirmation ID")?;

    eprintln!("Confirmation ID:");
    println!("{}", confirmation_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::confirmation::error::InstallationIdError;

    #[test]
    fn test_parse() {
        assert!(
            InstallationId::parse("334481558826870862843844566221823392794862457401103810", ActivationMode::Windows, None).is_ok()
        );
        assert!(
            InstallationId::parse("33448155882687086284384456622182339279486245740110381", ActivationMode::Windows, None)
                .is_err_and(|err| err == InstallationIdError::TooShort),
        );
        assert!(
            InstallationId::parse("3344815588268708628438445662218233927948624574011038100", ActivationMode::Windows, None)
                .is_err_and(|err| err == InstallationIdError::TooLong),
        );
        assert!(
            InstallationId::parse("33448155882687086284384456622182339279486245740110381!", ActivationMode::Windows, None)
                .is_err_and(|err| err == InstallationIdError::InvalidCharacter),
        );
        assert!(
            InstallationId::parse("334481558826870862843844566221823392794862457401103811", ActivationMode::Windows, None)
                .is_err_and(|err| err == InstallationIdError::InvalidCheckDigit),
        );
    }

    #[test]
    fn test_generate() {
        let iid = InstallationId::parse("334481558826870862843844566221823392794862457401103810", ActivationMode::Windows, None).unwrap();

        assert_eq!(
            generate(&iid).unwrap(),
            "110281-200130-887120-647974-697175-027544-252733"
        );
    }

    #[test]
    fn test_generate_v4() {
        let iid = InstallationId::parse("140360-627153-508674-221690-171243-904021-659581-150052-92", ActivationMode::Windows, None).unwrap();

        assert_eq!(
            generate(&iid).unwrap(),
            "109062-530373-462923-856922-378004-297663-022353"
        );
    }
}