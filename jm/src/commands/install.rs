use crate::npm::Fetcher;
use crate::package::Package;
use crate::resolver::Resolver;
use crate::workspace::WorkspacePackage;
use crate::Config;
use crate::Workspace;
use crate::Writer;

use array_tool::vec::*;
use futures::StreamExt;

const CONCURRENCY: usize = 8;

async fn get_workspace_package_tree(
    resolver: &Resolver,
    workspace_package: &WorkspacePackage,
) -> Result<(), String> {
    let mut list = vec![workspace_package.package.clone()];
    let mut seen = list.clone();

    while !list.is_empty() {
        let package = list.shift().unwrap();

        let dependencies_packages = futures::stream::iter(
            package
                .dependencies()
                .iter()
                .map(|dependency| resolver.get(&package.name, dependency)),
        )
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<Result<_, _>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<Package>, String>>()?;

        let new_packages: Vec<Package> = dependencies_packages
            .into_iter()
            .filter(|package| !seen.contains(package))
            .collect();

        seen.extend(new_packages.clone());
        list.extend(new_packages);
    }

    Ok(())
}

pub async fn install(config: &Config) -> Result<(), String> {
    let workspace = Workspace::from_config(config)?;
    let fetcher = Fetcher::new(config.registry.clone());
    let resolver = Resolver::new(fetcher);

    futures::stream::iter(
        workspace
            .workspace_packages
            .iter()
            .map(|workspace_package| get_workspace_package_tree(&resolver, &workspace_package)),
    )
    .buffer_unordered(CONCURRENCY)
    .collect::<Vec<Result<_, _>>>()
    .await
    .into_iter()
    .collect::<Result<Vec<()>, String>>()?;

    let _writer = Writer::new(&config)?;

    Ok(())
}
