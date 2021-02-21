use futures::StreamExt;
use log::debug;

use crate::npm::{Fetcher, PackageMetadata};
use crate::resolver::get_minimal_package_versions;
use crate::Config;
use crate::Workspace;
use crate::Writer;

pub async fn install(config: &Config) -> Result<(), String> {
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
