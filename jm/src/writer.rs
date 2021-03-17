use crate::config::Config;
use crate::downloader::Downloader;
use jm_core::errors::JmError;
use jm_core::package::NpmPackage;
use jm_core::package::Package;
use log::debug;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Dfs;

use std::fs;
use std::path::PathBuf;

pub struct Writer {
    store_path: PathBuf,
    downloader: Downloader,
}

impl Writer {
    pub fn new(config: &Config, downloader: Downloader) -> Result<Writer, JmError> {
        let store_path = config.root_path.as_path().join("node_modules").join(".jm");

        fs::create_dir_all(&store_path)?;

        Ok(Writer {
            store_path,
            downloader,
        })
    }

    pub fn write(
        &self,
        starting_nodes: Vec<NodeIndex>,
        graph: &Graph<Package, ()>,
    ) -> Result<(), JmError> {
        for node in starting_nodes {
            let mut dfs = Dfs::new(graph, node);
            while let Some(nx) = dfs.next(graph) {
                let package = &graph[nx];

                self.write_package(package)?;
            }
        }

        Ok(())
    }

    fn write_package(&self, package: &Package) -> Result<(), JmError> {
        match package {
            Package::Package(npm_package) => {
                let path = self.package_path(npm_package);
                debug!("Creating directory {:?}", &path);

                // TODO: test both scoped and non scoped packages
                fs::create_dir_all(&path)?;
                self.downloader.download_to(&npm_package.tarball_url, &path)?;
            }
            Package::WorkspacePackage(workspace_package) => {
                println!("Ignoring workspace package {:?}", workspace_package)
            }
        }

        Ok(())
    }

    fn package_path(&self, package: &NpmPackage) -> PathBuf {
        let package_dir_name = format!("{}@{}", package.name, package.version);

        self.store_path.join(package_dir_name)
    }
}
