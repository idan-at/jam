use futures::StreamExt;
use log::debug;

use crate::npm::{Fetcher, PackageMetadata};
use crate::resolver::get_minimal_package_versions;
use crate::Config;
use crate::Workspace;
use crate::Writer;

const CONCURRENCY: usize = 8;

pub async fn install(config: &Config) -> Result<(), String> {
    let workspace = Workspace::from_config(config)?;
    debug!("{:?}", workspace);

    let packages_requested_versions = workspace.collect_packages_versions();
    debug!("{:?}", packages_requested_versions);

    let fetcher = Fetcher::new(config.registry.clone());
    let packages_metadata: Vec<PackageMetadata> = futures::stream::iter(
        packages_requested_versions
            .keys()
            .map(|package_name| fetcher.get_package_metadata(package_name)),
    )
    .buffer_unordered(CONCURRENCY)
    .collect::<Vec<Result<_, _>>>()
    .await
    .into_iter()
    .collect::<Result<Vec<PackageMetadata>, String>>()?;

    let packages_versions_to_fetch =
        get_minimal_package_versions(packages_requested_versions, &packages_metadata);
    debug!("{:?}", packages_versions_to_fetch);

    let writer = Writer::new(&config);
    writer.init();

    Ok(())
}
