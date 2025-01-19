//! High-level commands for the Soldeer CLI
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
pub use crate::commands::{Args, Command};
use clap::builder::PossibleValue;
use cliclack::{intro, log::step, outro, outro_cancel};
use derive_more::derive::FromStr;
use soldeer_core::{config::Paths, Result};
use std::env;
use utils::get_config_location;

pub mod commands;
pub mod utils;

/// The location where the Soldeer config should be stored.
///
/// This is a new type so we can implement the `ValueEnum` trait for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromStr)]
pub struct ConfigLocation(soldeer_core::config::ConfigLocation);

impl clap::ValueEnum for ConfigLocation {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self(soldeer_core::config::ConfigLocation::Foundry),
            Self(soldeer_core::config::ConfigLocation::Soldeer),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self.0 {
            soldeer_core::config::ConfigLocation::Foundry => PossibleValue::new("foundry"),
            soldeer_core::config::ConfigLocation::Soldeer => PossibleValue::new("soldeer"),
        })
    }
}

impl From<ConfigLocation> for soldeer_core::config::ConfigLocation {
    fn from(value: ConfigLocation) -> Self {
        value.0
    }
}

impl From<soldeer_core::config::ConfigLocation> for ConfigLocation {
    fn from(value: soldeer_core::config::ConfigLocation) -> Self {
        Self(value)
    }
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::Init(cmd) => {
            intro("🦌 Soldeer Init 🦌")?;
            step("Initialize Foundry project to use Soldeer")?;
            let paths = Paths::with_config(Some(get_config_location(cmd.config_location)?))?;
            commands::init::init_command(&paths, cmd).await.inspect_err(|_| {
                outro_cancel("An error occurred during initialization").ok();
            })?;
            outro("Done initializing!")?;
        }
        Command::Install(cmd) => {
            intro("🦌 Soldeer Install 🦌")?;
            let paths = Paths::with_config(Some(get_config_location(cmd.config_location)?))?;
            commands::install::install_command(&paths, cmd).await.inspect_err(|_| {
                outro_cancel("An error occurred during install").ok();
            })?;
            outro("Done installing!")?;
        }
        Command::Update(cmd) => {
            intro("🦌 Soldeer Update 🦌")?;
            let paths = Paths::with_config(Some(get_config_location(cmd.config_location)?))?;
            commands::update::update_command(&paths, cmd).await.inspect_err(|_| {
                outro_cancel("An error occurred during the update").ok();
            })?;
            outro("Done updating!")?;
        }
        Command::Uninstall(cmd) => {
            intro("🦌 Soldeer Uninstall 🦌")?;
            let paths = Paths::with_config(Some(get_config_location(None)?))?;
            commands::uninstall::uninstall_command(&paths, &cmd).inspect_err(|_| {
                outro_cancel("An error occurred during uninstall").ok();
            })?;
            outro("Done uninstalling!")?;
        }
        Command::Login(cmd) => {
            intro("🦌 Soldeer Login 🦌")?;
            commands::login::login_command(cmd).await.inspect_err(|_| {
                outro_cancel("An error occurred during login").ok();
            })?;
            outro("Done logging in!")?;
        }
        Command::Push(cmd) => {
            intro("🦌 Soldeer Push 🦌")?;
            commands::push::push_command(cmd).await.inspect_err(|_| {
                outro_cancel("An error occurred during push").ok();
            })?;
            outro("Done!")?;
        }
        Command::Version(_) => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("soldeer {VERSION}");
        }
    }
    Ok(())
}
