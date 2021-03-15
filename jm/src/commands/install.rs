use crate::collector::Collector;
use crate::dependency::Dependency;
use crate::npm::Fetcher;
use crate::package::Package;
use crate::resolver::Resolver;
use crate::Config;
use crate::Workspace;
use crate::Writer;
use futures::StreamExt;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;

const CONCURRENCY: usize = 50;

pub async fn install(config: &Config) -> Result<(), String> {
    let workspace = Workspace::from_config(config)?;
    let fetcher = Fetcher::new(config.registry.clone());
    let resolver = Resolver::new(fetcher);
    let collector = Collector::new();
    let mut graph: Graph<Package, ()> = Graph::new();

    let mut list = workspace.packages();
    let mut seen: HashMap<Package, NodeIndex> = HashMap::new();

    list.iter().for_each(|package| {
        let node = graph.add_node(package.clone());
        seen.insert(package.clone(), node);
    });

    while !list.is_empty() {
        let dependencies_map = collector.collect(&list);

        let dependencies_packages = futures::stream::iter(
            dependencies_map
                .iter()
                .map(|(dependency, packages)| resolver.get(&packages[0].name, dependency)),
        )
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<Result<_, _>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<(Package, &Dependency)>, String>>()?;

        let new_packages: Vec<(Package, &Dependency)> = dependencies_packages
            .into_iter()
            .filter(|(package, _)| !seen.contains_key(package))
            .collect();

        new_packages.iter().for_each(|(package, _)| {
            let node = graph.add_node(package.clone());
            seen.insert(package.clone(), node);
        });

        new_packages.iter().for_each(|(package, dependency)| {
            let parent_nodes = dependencies_map
                .get(dependency)
                .unwrap()
                .iter()
                .map(|package| seen.get(package).unwrap());
            let node = seen.get(package).unwrap();

            parent_nodes.for_each(|parent_node| {
                graph.add_edge(*parent_node, *node, ());
            })
        });

        list = new_packages
            .iter()
            .map(|(package, _)| package.clone())
            .collect();
    }

    let writer = Writer::new(&config)?;

    writer.write(&graph)?;

    Ok(())
}
