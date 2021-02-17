mod common;
mod config;
mod npm;
mod resolver;
mod workspace;
mod writer;

use futures::StreamExt;
use log::debug;
use std::path::PathBuf;

use common::read_manifest_file;
use config::Config;
use npm::{Fetcher, PackageMetadata};
use resolver::get_minimal_package_versions;
use workspace::Workspace;
use writer::Writer;

pub async fn install(root_path: PathBuf) -> Result<(), String> {
    let manifest_file_path = root_path.join("package.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content)?;
    debug!("{:?}", config);

    let workspace = Workspace::from_config(&config)?;
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
