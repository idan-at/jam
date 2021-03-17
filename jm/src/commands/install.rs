use crate::downloader::Downloader;
use crate::npm::Fetcher;
use crate::resolver::Resolver;
use crate::Config;
use crate::Workspace;
use crate::Writer;
use jm_core::build_graph;
use jm_core::errors::JmError;

pub async fn install(config: &Config) -> Result<(), JmError> {
    let workspace = Workspace::from_config(config)?;
    let fetcher = Fetcher::new(config.registry.clone());
    let resolver = Resolver::new(fetcher);

    let (starting_nodes, graph) = build_graph(workspace.packages(), &resolver).await?;

    let downloader = Downloader::new();
    let writer = Writer::new(&config, downloader)?;

    writer.write(starting_nodes, &graph).await?;

    Ok(())
}
