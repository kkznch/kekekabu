use anyhow::Result;

use crate::config;
use crate::spec;

pub fn init(force: bool) -> Result<()> {
    config::init_config(force)
}

pub fn validate() -> Result<()> {
    let config = config::AppConfig::load()?;
    eprintln!("Config: OK");

    let spec_result = spec::load_spec(&config.spec.path);
    match spec_result {
        Ok(s) => eprintln!("Spec ({}): OK", s.name),
        Err(e) => {
            eprintln!("Spec: FAILED");
            return Err(e);
        }
    }

    eprintln!("All validations passed.");
    Ok(())
}
