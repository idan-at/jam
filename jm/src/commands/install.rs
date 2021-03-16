use crate::npm::Fetcher;
use crate::resolver::Resolver;
use crate::Config;
use crate::Workspace;
use crate::Writer;
use jm_core::build_graph;

pub async fn install(config: &Config) -> Result<(), String> {
    let workspace = Workspace::from_config(config)?;
    let fetcher = Fetcher::new(config.registry.clone());
    let resolver = Resolver::new(fetcher);

    let graph = build_graph(workspace.packages(), &resolver).await?;

    let writer = Writer::new(&config)?;

    writer.write(&graph)?;

    Ok(())
}
