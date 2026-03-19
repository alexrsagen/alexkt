mod activate_xp;
mod bink_info;
mod generate_confirmation;
mod generate_keys_pidgen_v2;
mod generate_keys_pidgen_v3;

use crate::App;
use crate::args;
use anyhow::Result;

pub fn execute(app: &App) -> Result<()> {
    match &app.args.cmd {
        Some(args::Command::BinkInfo { file_path }) => bink_info::execute(app, file_path.as_deref()),
        Some(args::Command::GeneratePidgen2 { variant }) => generate_keys_pidgen_v2::execute(app, *variant),
        Some(args::Command::GeneratePidgen3 { version, bink_id, channel_id, upgrade }) => generate_keys_pidgen_v3::execute(app, *version, *bink_id, *channel_id, *upgrade),
        Some(args::Command::GenerateConfirmation { installation_id, mode, product_id }) => generate_confirmation::execute(app, installation_id, *mode, *product_id),
        Some(args::Command::ActivateXP { force }) => activate_xp::execute(app, *force),
        None => activate_xp::execute(app, false),
    }
}
