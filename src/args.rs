use std::path::PathBuf;

use clap::{Parser, Subcommand};
use num_traits::{Unsigned, Num};

use crate::confirmation::{ActivationMode, ProductId};
use crate::pidgen::v2::KeyVariant as KeyVariantV2;
use crate::pidgen::v3::KeyVersion as KeyVersionV3;

pub fn maybe_hex<T: Num + Unsigned>(s: &str) -> Result<T, T::FromStrRadixErr>
where
    <T as Num>::FromStrRadixErr: std::fmt::Display,
{
    const HEX_PREFIX: &str = "0x";
    const HEX_PREFIX_UPPER: &str = "0X";
    const HEX_PREFIX_LEN: usize = HEX_PREFIX.len();

    if s.starts_with(HEX_PREFIX) || s.starts_with(HEX_PREFIX_UPPER) {
        T::from_str_radix(&s[HEX_PREFIX_LEN..], 16)
    } else {
        T::from_str_radix(s, 10)
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(alias = "bink")]
    BinkInfo {
        /// Path to pidgen.dll or BINK file (pidgen.dll resource BINK/1-2 or *.pub)
        #[clap(long)]
        file_path: Option<PathBuf>,
    },

    #[clap(alias = "gen2")]
    GeneratePidgen2 {
        /// Key variant: Can be Retail, Office or OEM
        variant: KeyVariantV2,
    },

    #[clap(alias = "gen3")]
    GeneratePidgen3 {
        /// Key version: Can be Bink1998 or Bink2002
        #[clap(long, default_value = "bink1998")]
        version: KeyVersionV3,
        /// BINK ID
        #[clap(long, value_parser = maybe_hex::<u32>)]
        bink_id: u32,
        /// Channel ID
        #[clap(long, value_parser = maybe_hex::<u32>)]
        channel_id: u32,
        /// Whether the key is an upgrade key
        #[clap(long, default_value = "false")]
        upgrade: bool,
    },

    #[clap(alias = "confirm")]
    GenerateConfirmation {
        /// Installation ID string
        installation_id: String,
        /// Activation mode: Can be Windows, OfficeXP, Office2003, Office2007 or PlusDigitalMediaEdition
        #[clap(long, default_value = "windows")]
        mode: ActivationMode,
        /// Product ID string (XXXXX-XXX-XXXXXXX-XXXXX)
        #[clap(long)]
        product_id: Option<ProductId>,
    },
}

#[derive(Debug, Parser)]
#[clap(author, about, version)]
pub struct Args {
    /// Log level [off|error|warn|info|debug|trace]
    #[clap(long, short = 'l', default_value = "info")]
    pub log_level: log::LevelFilter,

    /// Path to UMSKT-style key database (JSON)
    #[clap(long)]
    pub keydb: Option<PathBuf>,

    /// Command
    #[clap(subcommand)]
    pub cmd: Command,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
