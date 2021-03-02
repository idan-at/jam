pub mod cli_opts;
pub mod npm;

mod commands;
mod common;
mod config;
mod package;
mod resolver;
mod root_locator;
mod workspace;
mod writer;
mod dependency;

use crate::cli_opts::Opts;
use log::debug;
use std::path::PathBuf;

use cli_opts::Command;
use commands::install::install;
use common::read_manifest_file;
use config::Config;
use root_locator::find_root_dir;
use workspace::Workspace;
use writer::Writer;

pub async fn run(cwd: PathBuf, opts: Opts) -> Result<(), String> {
    let root_path = find_root_dir(cwd)?;
    debug!("Root path {:?}", root_path);

    let manifest_file_path = root_path.join("jm.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content, &opts.registry)?;
    debug!("Config {:?}", config);

    match opts.command {
        Command::Install(_) | Command::I(_) => install(&config).await,
    }
}
