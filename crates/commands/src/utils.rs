//! Utils for the commands crate
use std::fmt;

use crate::ConfigLocation;
use cliclack::{multi_progress, progress_bar, select, MultiProgress, ProgressBar};
use soldeer_core::{
    config::{detect_config_location, Paths},
    install::InstallMonitoring,
    Result,
};

/// Template for the progress bars.
pub const PROGRESS_TEMPLATE: &str = "[{elapsed_precise}] {bar:30.magenta} ({pos}/{len}) {msg}";

/// A collection of progress bars for the installation/update process.
#[derive(Clone)]
pub struct Progress {
    multi: MultiProgress,
    versions: ProgressBar,
    downloads: ProgressBar,
    unzip: ProgressBar,
    subdependencies: ProgressBar,
    integrity: ProgressBar,
}

impl Progress {
    /// Create a new progress bar object.
    ///
    /// A title and the total number of dependencies to install must be passed as an argument.
    pub fn new(title: impl fmt::Display, total: usize, mut monitor: InstallMonitoring) -> Self {
        let multi = multi_progress(title);
        let versions = multi.add(progress_bar(total as u64).with_template(PROGRESS_TEMPLATE));
        let downloads = multi.add(progress_bar(total as u64).with_template(PROGRESS_TEMPLATE));
        let unzip = multi.add(progress_bar(total as u64).with_template(PROGRESS_TEMPLATE));
        let subdependencies =
            multi.add(progress_bar(total as u64).with_template(PROGRESS_TEMPLATE));
        let integrity = multi.add(progress_bar(total as u64).with_template(PROGRESS_TEMPLATE));
        tokio::task::spawn({
            let multi = multi.clone();
            async move {
                while let Some(log) = monitor.logs.recv().await {
                    multi.println(log);
                }
            }
        });
        tokio::task::spawn({
            let versions = versions.clone();
            async move {
                while let Some(dep) = monitor.versions.recv().await {
                    versions.inc(1);
                    versions.set_message(format!("Got version for {dep}"));
                }
            }
        });
        tokio::task::spawn({
            let downloads = downloads.clone();
            async move {
                while let Some(dep) = monitor.downloads.recv().await {
                    downloads.inc(1);
                    downloads.set_message(format!("Downloaded {dep}"));
                }
            }
        });
        tokio::task::spawn({
            let unzip = unzip.clone();
            async move {
                while let Some(dep) = monitor.unzip.recv().await {
                    unzip.inc(1);
                    unzip.set_message(format!("Unzipped {dep}"));
                }
            }
        });
        tokio::task::spawn({
            let subdependencies = subdependencies.clone();
            async move {
                while let Some(dep) = monitor.subdependencies.recv().await {
                    subdependencies.inc(1);
                    subdependencies.set_message(format!("Installed subdeps for {dep}"));
                }
            }
        });
        tokio::task::spawn({
            let integrity = integrity.clone();
            async move {
                while let Some(dep) = monitor.integrity.recv().await {
                    integrity.inc(1);
                    integrity.set_message(format!("Checked integrity of {dep}"));
                }
            }
        });
        Self { multi, versions, downloads, unzip, subdependencies, integrity }
    }

    /// Start all progress bars.
    pub fn start_all(&self) {
        self.versions.start("Retrieving versions...");
        self.downloads.start("Downloading dependencies...");
        self.unzip.start("Unzipping dependencies...");
        self.subdependencies.start("Installing subdependencies...");
        self.integrity.start("Checking integrity...");
    }

    /// Stop all progress bars.
    pub fn stop_all(&self) {
        self.versions.stop("Done retrieving versions");
        self.downloads.stop("Done downloading dependencies");
        self.unzip.stop("Done unzipping dependencies");
        self.subdependencies.stop("Done installing subdependencies");
        self.integrity.stop("Done checking integrity");
        self.multi.stop();
    }

    pub fn set_error(&self, error: impl fmt::Display) {
        self.multi.error(error);
    }
}

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
