use anyhow::Result;
use chrono::Local;
use fern::Output;
use log::LevelFilter;
use std::{env, str::FromStr};

///Setup the default logger, this function take an Output in entry,
///#Example
///```
/// setup_logger(std::io::stdout()).unwrap()
///
///```
#[cfg(not(tarpaulin_include))]
pub fn setup_logger<T>(_output: T) -> Result<()>
where
    T: Into<Output>,
{
    {
        let filter = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(LevelFilter::from_str(&filter).unwrap_or(LevelFilter::Info))
            .level_for("hyper", log::LevelFilter::Info)
            .level_for("_", log::LevelFilter::Warn)
            .chain(_output)
            .apply()?
    }
    Ok(())
}
