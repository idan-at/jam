mod common;
mod config;
mod npm;
mod workspace;

use env_logger;
use log::debug;
use std::path::PathBuf;

use common::read_manifest_file;
use config::Config;
use npm::PackageMetadata;
use workspace::Workspace;

pub fn run(root_path: PathBuf) -> Result<(), String> {
    let _ = env_logger::try_init();

    let manifest_file_path = root_path.join("package.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content)?;
    debug!("{:?}", config);

    let workspace = Workspace::from_config(&config)?;
    debug!("{:?}", workspace);

    let packages_versions = workspace.collect_packages_versions();
    debug!("{:?}", packages_versions);

    let package_metadata: Vec<PackageMetadata> = packages_versions
        .keys()
        .map(|package_name| npm::get_package_metadata(package_name))
        .collect();

    println!("{:?}", package_metadata);

    Ok(())
}
