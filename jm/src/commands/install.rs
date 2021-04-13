use crate::archiver::DefaultArchiver;
use crate::downloader::TarDownloader;
use crate::resolver::Resolver;
use crate::Config;
use crate::JmError;
use crate::Workspace;
use crate::Writer;
use directories::ProjectDirs;
use jm_cache::CacheFactory;
use jm_core::build_graph;
use jm_core::npm::Fetcher;

pub async fn install(config: &Config) -> Result<(), JmError> {
    let workspace = Workspace::from_config(config)?;
    let project_dirs =
        ProjectDirs::from("com", "jm", &config.cache_group).expect("Failed to locate project dir");
    let cache_factory = CacheFactory::new(project_dirs.cache_dir().to_path_buf());
    let fetcher = Fetcher::new(&cache_factory, &config.registry)?;
    let resolver = Resolver::new(fetcher);

    let (starting_nodes, graph) = build_graph(workspace.packages(), &resolver).await?;

    let archiver = DefaultArchiver::new();
    let downloader = TarDownloader::new(&cache_factory, &archiver)?;
    let writer = Writer::new(project_dirs.data_dir(), &downloader)?;

    writer.write(starting_nodes, &graph).await?;

    Ok(())
}
