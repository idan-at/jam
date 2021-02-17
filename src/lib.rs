mod common;
mod config;
mod npm;
mod resolver;
mod workspace;

use futures::StreamExt;
use env_logger;
use log::debug;
use std::path::PathBuf;

use common::read_manifest_file;
use config::Config;
use npm::{NpmFacade, PackageMetadata};
use resolver::get_minimal_package_versions;
use workspace::Workspace;

pub async fn install(root_path: PathBuf) -> Result<(), String> {
    let _ = env_logger::try_init();

    let manifest_file_path = root_path.join("package.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content)?;
    debug!("{:?}", config);

    let workspace = Workspace::from_config(&config)?;
    debug!("{:?}", workspace);

    let packages_requested_versions = workspace.collect_packages_versions();
    debug!("{:?}", packages_requested_versions);

    let npm_facade = NpmFacade::new();
    let packages_metadata: Vec<PackageMetadata> = futures::stream::iter(
        packages_requested_versions
            .keys()
            .map(|package_name| npm_facade.get_package_metadata(package_name))
    ).buffer_unordered(8).collect().await;
    debug!("{:?}", packages_metadata);

    let packages_versions_to_fetch =
        get_minimal_package_versions(packages_requested_versions, &packages_metadata);

    println!("{:?}", packages_versions_to_fetch);

    Ok(())
}
