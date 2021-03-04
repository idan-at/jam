use crate::collector::Collector;
use crate::npm::Fetcher;
use crate::package::Package;
use crate::resolver::Resolver;
use crate::Config;
use crate::Workspace;
use crate::Writer;
use futures::StreamExt;

const CONCURRENCY: usize = 50;

pub async fn install(config: &Config) -> Result<(), String> {
    let workspace = Workspace::from_config(config)?;
    let fetcher = Fetcher::new(config.registry.clone());
    let resolver = Resolver::new(fetcher);
    let collector = Collector::new();

    let mut list = workspace.packages();
    let mut seen = list.clone();

    while !list.is_empty() {
        let dependencies = collector.collect(&list);

        list.clear();

        let dependencies_packages = futures::stream::iter(
            dependencies
                .iter()
                .map(|dependency| resolver.get(/* &package.name */ "something", dependency)),
        )
        .buffer_unordered(CONCURRENCY * 3)
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

    let _writer = Writer::new(&config)?;

    Ok(())
}
