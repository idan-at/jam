use crate::package::to_dependencies_hash_map;
use crate::package::Dependency;
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
    // let dependencies = package.dependencies();

    // let packages_metadata: HashMap<String, PackageMetadata> =
    //     get_packages_metadata(fetcher, &package.name, &dependencies)
    //         .await?
    //         .into_iter()
    //         .map(|package_metadata| (package_metadata.package_name.to_string(), package_metadata))
    //         .collect();

    for (dependency_name, dependency) in package.dependencies() {
        let real_name = dependency.real_name;
        let version_or_dist_tag = dependency.version_or_dist_tag;

        println!("{} {}", real_name, version_or_dist_tag);

        // let metadata = packages_metadata.get(&dependency_name).unwrap();
        let metadata = get_package_metadata(fetcher, &package.name, &real_name).await?;
        let version = get_package_exact_version(&dependency_name, &version_or_dist_tag, &metadata);

        let version_metadata = metadata.versions.get(&version).unwrap();

        let package = Package {
            name: dependency_name.to_string(),
            version: version.clone(),
            dependencies: to_dependencies_hash_map(Some(version_metadata.dependencies.clone())),
            dev_dependencies: to_dependencies_hash_map(Some(version_metadata.dev_dependencies.clone())),
        };

        let node = graph.add_node(PackageNode {
            name: dependency_name.to_string(),
            version,
        });

        graph.add_edge(parent, node, ());

        new_nodes.push((node, package));
    }

    Ok(new_nodes)
}

// async fn get_packages_metadata(
//     fetcher: &Fetcher,
//     package_name: &str,
//     dependencies: &HashMap<String, Dependency>,
// ) -> Result<Vec<PackageMetadata>, String> {
//     // TODO: use real name for fetching metadata
//     match futures::stream::iter(
//         dependencies
//             .keys()
//             .map(|package_name| fetcher.get_package_metadata(package_name)),
//     )
//     .buffer_unordered(CONCURRENCY)
//     .collect::<Vec<Result<_, _>>>()
//     .await
//     .into_iter()
//     .collect::<Result<Vec<PackageMetadata>, String>>()
//     {
//         Ok(metadata) => Ok(metadata),
//         Err(err) => Err(format!("{}->{}", package_name, err)),
//     }
// }

async fn get_package_metadata(
    fetcher: &Fetcher,
    parent: &str,
    package_name: &str,
) -> Result<PackageMetadata, String> {
    match fetcher.get_package_metadata(package_name).await {
        Ok(metadata) => Ok(metadata),
        Err(err) => Err(format!("{}->{}", parent, err)),
    }
}
