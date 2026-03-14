use core::fmt;

use anyhow::{Context, Result};
use env_logger::Builder;
use log::LevelFilter;

pub fn try_init(level: LevelFilter) -> Result<()> {
    let mut builder = Builder::new();

    builder.filter_level(level);

    builder.format(move |f, record| {
        use std::io::Write;
        let target = record.target();

        let time = f.timestamp_millis();

        writeln!(
            f,
            "[{}][{}][{}] {}",
            time,
            level,
            target,
            record.args(),
        )
    });

    builder.try_init().context("could not initialize logger")?;
    Ok(())
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}
