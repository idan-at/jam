pub mod cli_opts;

mod common;
mod config;
mod npm;
mod resolver;
mod root_locator;
mod workspace;
mod writer;

use crate::cli_opts::Opts;
use futures::StreamExt;
use log::debug;
use std::path::PathBuf;

use common::read_manifest_file;
use config::Config;
use npm::{Fetcher, PackageMetadata};
use resolver::get_minimal_package_versions;
use root_locator::find_root_dir;
use workspace::Workspace;
use writer::Writer;

pub async fn run(cwd: PathBuf, opts: Opts) -> Result<(), String> {
    let root_path = find_root_dir(cwd)?;
    debug!("Root path {:?}", root_path);

    let manifest_file_path = root_path.join("jm.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content)?;
    debug!("{:?}", config);

    match opts.command {
        _ => install(&config).await,
    }
}

async fn install(config: &Config) -> Result<(), String> {
    let workspace = Workspace::from_config(config)?;
    debug!("{:?}", workspace);

    let packages_requested_versions = workspace.collect_packages_versions();
    debug!("{:?}", packages_requested_versions);

    let fetcher = Fetcher::new();
    let packages_metadata: Vec<PackageMetadata> = futures::stream::iter(
        packages_requested_versions
            .keys()
            .map(|package_name| fetcher.get_package_metadata(package_name)),
    )
    // TODO: make 8 configurable
    .buffer_unordered(8)
    .collect()
    .await;

    let packages_versions_to_fetch =
        get_minimal_package_versions(packages_requested_versions, &packages_metadata);
    debug!("{:?}", packages_versions_to_fetch);

    let writer = Writer::new(&config);
    writer.init();

    Ok(())
}
