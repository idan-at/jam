use crate::npm::{Fetcher, PackageMetadata};
use crate::package::{Package, PackageNode};
use crate::resolver::get_package_exact_version;
use crate::Config;
use crate::Workspace;
use crate::Writer;

use array_tool::vec::*;
use futures::StreamExt;
use log::debug;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;

const CONCURRENCY: usize = 8;

pub async fn install(config: &Config) -> Result<(), String> {
    let workspace = Workspace::from_config(config)?;
    let fetcher = Fetcher::new(config.registry.clone());

    let mut graph: Graph<PackageNode, ()> = Graph::new();

    let mut list = workspace
        .workspace_packages
        .iter()
        .map(|workspace_package| {
            (
                graph.add_node(PackageNode {
                    name: workspace_package.package.name.clone(),
                    version: workspace_package.package.version.clone(),
                }),
                workspace_package.package.clone(),
            )
        })
        .collect::<Vec<_>>();

    while !list.is_empty() {
        let (parent, package) = list.shift().unwrap();

        let new_nodes = step(&mut graph, parent, &fetcher, &package).await?;
        list.extend(new_nodes);
    }

    debug!("graph {:?}", graph);

    let _writer = Writer::new(&config)?;

    Ok(())
}

async fn step(
    graph: &mut Graph<PackageNode, ()>,
    parent: NodeIndex,
    fetcher: &Fetcher,
    package: &Package,
) -> Result<Vec<(NodeIndex, Package)>, String> {
    let mut new_nodes = Vec::<(NodeIndex, Package)>::new();
    // TODO: warning when package has a dependency that is also a dev dependency
    let dependencies: HashMap<String, String> = package
        .dependencies
        .clone()
        .into_iter()
        .chain(package.dev_dependencies.clone())
        .collect();

    let packages_metadata: HashMap<String, PackageMetadata> =
        get_packages_metadata(fetcher, &package.name, dependencies.clone())
            .await?
            .into_iter()
            .map(|package_metadata| (package_metadata.package_name.to_string(), package_metadata))
            .collect();

    for (dependency_name, requested_version) in dependencies {
        let metadata = packages_metadata.get(&dependency_name).unwrap();
        let version = get_package_exact_version(&dependency_name, &requested_version, &metadata);

        let version_metadata = metadata.versions.get(&version).unwrap();

        let package = Package {
            name: dependency_name.clone(),
            version: version.clone(),
            dependencies: version_metadata.dependencies.clone(),
            dev_dependencies: version_metadata.dev_dependencies.clone(),
        };

        let node = graph.add_node(PackageNode {
            name: dependency_name,
            version,
        });

        graph.add_edge(parent, node, ());

        new_nodes.push((node, package));
    }

    Ok(new_nodes)
}

async fn get_packages_metadata(
    fetcher: &Fetcher,
    package_name: &str,
    dependencies: HashMap<String, String>,
) -> Result<Vec<PackageMetadata>, String> {
    match futures::stream::iter(
        dependencies
            .keys()
            .map(|package_name| fetcher.get_package_metadata(&package_name)),
    )
    .buffer_unordered(CONCURRENCY)
    .collect::<Vec<Result<_, _>>>()
    .await
    .into_iter()
    .collect::<Result<Vec<PackageMetadata>, String>>()
    {
        Ok(metadata) => Ok(metadata),
        Err(err) => Err(format!("{}->{}", package_name, err)),
    }
}
