use crate::config::Config;
use jm_core::package::NpmPackage;
use jm_core::package::Package;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Dfs;

use std::fs;
use std::path::PathBuf;

pub struct Writer {
    store_path: PathBuf,
}

impl Writer {
    pub fn new(config: &Config) -> Result<Writer, String> {
        let store_path = config.root_path.as_path().join("node_modules").join(".jm");

        match fs::create_dir_all(&store_path) {
            Ok(_) => Ok(Writer { store_path }),
            Err(err) => Err(String::from(err.to_string())),
        }
    }

    pub fn write(
        &self,
        starting_nodes: Vec<NodeIndex>,
        graph: &Graph<Package, ()>,
    ) -> Result<(), String> {
        for node in starting_nodes {
            let mut dfs = Dfs::new(graph, node);
            while let Some(nx) = dfs.next(graph) {
                let package = &graph[nx];

                self.write_package(package)?;
            }
        }

        Ok(())
    }

    fn write_package(&self, package: &Package) -> Result<(), String> {
        match package {
            Package::Package(npm_package) => {
                let path = self.package_path(npm_package);

                match fs::create_dir(&path) {
                    Ok(_) => {}
                    Err(err) => return Err(String::from(err.to_string())),
                }
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
