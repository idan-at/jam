pub mod cli_options;
pub mod errors;

mod archiver;
mod commands;
mod common;
mod config;
mod downloader;
mod resolver;
mod root_locator;
mod workspace;
mod writer;

use crate::cli_options::CliOptions;
use crate::errors::JmError;
use log::debug;
use std::path::PathBuf;

use cli_options::Command;
use commands::install::install;
use common::read_manifest_file;
use config::Config;
use root_locator::find_root_dir;
use workspace::Workspace;
use writer::Writer;

pub async fn run(cwd: PathBuf, options: CliOptions) -> Result<(), JmError> {
    let root_path = find_root_dir(cwd)?;
    debug!("Root path {:?}", root_path);

    let manifest_file_path = root_path.join("jm.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content, &options.registry)?;
    debug!("Config {:?}", config);

    match options.command {
        Command::Install(_) | Command::I(_) => install(&config).await,
    }
}
