pub mod cli_options;
pub mod errors;

mod archiver;
mod commands;
mod common;
mod config;
mod downloader;
mod resolver;
mod root_locator;
mod store;
mod workspace;
mod writer;

use crate::cli_options::CliOptions;
use crate::errors::JamError;
use cli_options::Command;
use commands::install::install;
use common::read_manifest_file;
use config::Config;
use directories::ProjectDirs;
use log::debug;
use root_locator::find_root_dir;
use std::path::PathBuf;
use workspace::Workspace;
use writer::Writer;

pub async fn run(cwd: PathBuf, options: CliOptions) -> Result<(), JamError> {
    let root_path = find_root_dir(cwd)?;
    debug!("Root path {:?}", root_path);

    let manifest_file_path = root_path.join("jam.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content, &options.registry)?;
    debug!("Config {:?}", config);

    let project_dirs = ProjectDirs::from("com", "jam", &options.cache_group)
        .expect("Failed to locate project dir");
    debug!("Project Dirs {:?}", config);

    match options.command {
        Command::Install(_) | Command::I(_) => install(&config, &project_dirs).await,
    }
}
