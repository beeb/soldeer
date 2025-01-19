//! Utils for the commands crate
use crate::ConfigLocation;
use cliclack::select;
use soldeer_core::{
    config::{detect_config_location, Paths},
    Result,
};

/// Auto-detect config location or prompt the user for preference.
pub fn get_config_location(
    arg: Option<ConfigLocation>,
) -> Result<soldeer_core::config::ConfigLocation> {
    Ok(match arg {
        Some(loc) => loc.into(),
        None => match detect_config_location(Paths::get_root_path()) {
            Some(loc) => loc,
            None => prompt_config_location()?.into(),
        },
    })
}

/// Prompt the user for their desired config location in case it cannot be auto-detected.
pub fn prompt_config_location() -> Result<ConfigLocation> {
    Ok(select("Select how you want to configure Soldeer")
        .initial_value("foundry")
        .item("foundry", "Using foundry.toml", "recommended")
        .item("soldeer", "Using soldeer.toml", "for non-foundry projects")
        .interact()?
        .parse()
        .expect("all options should be valid variants of the ConfigLocation enum"))
}
