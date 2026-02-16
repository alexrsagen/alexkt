mod args;
mod base24;
mod bytesize;
mod confirmation;
mod cmd;
mod error;
mod keydb;
mod logger;
mod crypto;
mod bink;
mod pidgen;
mod serde_bigint;

use anyhow::Result;

pub struct App {
    pub args: args::Args,
    pub keys: keydb::Keys,
    pub base24: base24::Base24,
}

fn main() -> Result<()> {
    // parse command-line arguments
    let args = args::Args::parse();

    // set up logger
    logger::try_init(args.log_level)?;

    // initialize key database
    let keys = if let Some(path) = &args.keydb {
        keydb::load_keys(path)?
    } else {
        keydb::load_default_keys()?
    };

    // initialize base24 encoder/decoder
    let base24 = base24::Base24::with_alphabet(base24::Base24::ALPHABET_MS)?;

    // execute command
    let app = App { args, keys, base24 };
    cmd::execute(&app)
}
