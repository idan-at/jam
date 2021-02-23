use crate::npm::{Fetcher, PackageMetadata};
use crate::package::to_dependencies_list;
use crate::package::Dependency;
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

async fn step2(fetcher: &Fetcher, package_name: &str, dependency: &Dependency) -> Result<(PackageNode, Package), String> {
    let metadata = get_package_metadata(fetcher, package_name, &dependency.real_name).await?;
    let version = get_package_exact_version(
        package_name,
        &dependency.name,
        &dependency.version_or_dist_tag,
        &metadata,
    );

    let version_metadata = metadata.versions.get(&version).unwrap();

    let package = Package {
        name: dependency.name.to_string(),
        version: version.clone(),
        dependencies: to_dependencies_list(Some(version_metadata.dependencies.clone())),
        dev_dependencies: to_dependencies_list(Some(
            version_metadata.dev_dependencies.clone(),
        )),
    };

    Ok((PackageNode {
        name: dependency.name.to_string(),
        version,
    }, package))
}

async fn step(
    graph: &mut Graph<PackageNode, ()>,
    parent: NodeIndex,
    fetcher: &Fetcher,
    package: &Package,
) -> Result<Vec<(NodeIndex, Package)>, String> {
    let mut new_nodes = Vec::<(NodeIndex, Package)>::new();

    let collected_packages = futures::stream::iter(package.dependencies().iter().map(|dependency| {
        step2(fetcher, &package.name, dependency)
    })).buffer_unordered(CONCURRENCY).collect::<Vec<Result<_,_>>>().await.into_iter()
        .collect::<Result<Vec<(PackageNode, Package)>, String>>()?;

    collected_packages.into_iter().for_each(|(package_node, package)| {
        let node = graph.add_node(package_node);
        graph.add_edge(parent, node, ());
        new_nodes.push((node, package));
    });

    Ok(new_nodes)
}

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
