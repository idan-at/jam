mod common;
mod config;
mod npm;
mod versions;
mod workspace;

use env_logger;
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;

use common::read_manifest_file;
use config::Config;
use npm::PackageMetadata;
use versions::get_minimal_package_versions;
use workspace::Workspace;

pub fn run(root_path: PathBuf) -> Result<(), String> {
    let _ = env_logger::try_init();

    let manifest_file_path = root_path.join("package.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content)?;
    debug!("{:?}", config);

    let workspace = Workspace::from_config(&config)?;
    debug!("{:?}", workspace);

    let packages_requested_versions = workspace.collect_packages_versions();

    let packages_metadata: HashMap<String, PackageMetadata> = packages_requested_versions
        .keys()
        .map(|package_name| {
            (
                package_name.to_string(),
                npm::get_package_metadata(package_name),
            )
        })
        .collect();

    let packages_versions_to_fetch =
        get_minimal_package_versions(packages_requested_versions, &packages_metadata);

    println!("{:?}", packages_versions_to_fetch);

    Ok(())
}
