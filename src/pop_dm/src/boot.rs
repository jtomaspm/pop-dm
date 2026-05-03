use crate::config::Config;
use pop_dm_lib::error::Result;

pub fn boot() -> Result<Config> {
    Ok(Config::default())
}

pub fn run(config: Config) -> Result<()> {
    return Ok(());
}

pub fn boot_and_run() -> Result<()> {
    return run(boot()?);
}
