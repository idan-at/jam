use crate::archiver::DefaultArchiver;
use crate::downloader::TarDownloader;
use crate::resolver::Resolver;
use crate::Config;
use crate::JmError;
use crate::Workspace;
use crate::Writer;
use jm_core::npm::Fetcher;
use jm_core::build_graph;

pub async fn install(config: &Config) -> Result<(), JmError> {
    let workspace = Workspace::from_config(config)?;
    let fetcher = Fetcher::new(config.cache_group.clone(), &config.registry)?;
    let resolver = Resolver::new(fetcher);

    let (starting_nodes, graph) = build_graph(workspace.packages(), &resolver).await?;

    let archiver = DefaultArchiver::new();
    let downloader = TarDownloader::new(config.cache_group.clone(), &archiver)?;
    let writer = Writer::new(config.root_path.as_path(), &downloader)?;

    writer.write(starting_nodes, &graph).await?;

    Ok(())
}
