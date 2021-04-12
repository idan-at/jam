use crate::common::sanitize_package_name;
use crate::downloader::Downloader;
use jm_core::errors::JmError;
use jm_core::package::NpmPackage;
use jm_core::package::Package;
use log::debug;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Dfs;
use std::path::Path;

use std::fs;
use std::path::PathBuf;

pub struct Writer<'a> {
    store_path: PathBuf,
    downloader: &'a dyn Downloader,
}

impl<'a> Writer<'a> {
    pub fn new(root_path: &Path, downloader: &'a dyn Downloader) -> Result<Writer<'a>, JmError> {
        let store_path = root_path.join("node_modules").join(".jm");

        fs::create_dir_all(&store_path)?;

        Ok(Writer {
            store_path,
            downloader,
        })
    }

    pub async fn write(
        &self,
        starting_nodes: Vec<NodeIndex>,
        graph: &Graph<Package, ()>,
    ) -> Result<(), JmError> {
        for node in starting_nodes {
            let mut dfs = Dfs::new(graph, node);
            while let Some(nx) = dfs.next(graph) {
                let package = &graph[nx];

                self.write_package(package).await?;
            }
        }

        Ok(())
    }

    async fn write_package(&self, package: &Package) -> Result<(), JmError> {
        match package {
            Package::NpmPackage(npm_package) => {
                let path = self.package_path(npm_package);
                debug!("Creating directory {:?}", &path);

                fs::create_dir(&path)?;
                self.downloader.download_to(&npm_package, &path).await?;
            }
            Package::WorkspacePackage(workspace_package) => {
                debug!("Ignoring workspace package {:?}", workspace_package)
            }
        }

        Ok(())
    }

    fn package_path(&self, package: &NpmPackage) -> PathBuf {
        let package_dir_name = format!(
            "{}@{}",
            sanitize_package_name(&package.name),
            package.version
        );

        self.store_path.join(package_dir_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archiver::DefaultArchiver;
    use crate::downloader::TarDownloader;
    use async_trait::async_trait;
    use jm_core::package::WorkspacePackage;
    use maplit::hashmap;
    use tempdir::TempDir;

    fn create_graph() -> (NodeIndex, Graph<Package, ()>) {
        let npm_package = Package::NpmPackage(NpmPackage::new(
            "p1".to_string(),
            "1.0.0".to_string(),
            None,
            "shasum".to_string(),
            "tarball-url".to_string(),
        ));
        let scoped_npm_package = Package::NpmPackage(NpmPackage::new(
            "@scope/p1".to_string(),
            "2.0.0".to_string(),
            None,
            "shasum".to_string(),
            "tarball-url".to_string(),
        ));
        let workspace_package = Package::WorkspacePackage(WorkspacePackage::new(
            "workspace_package".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
                "p1".to_string() => "1.0.0".to_string(),
                "@scope/p1".to_string() => "2.0.0".to_string(),
            }),
            None,
            PathBuf::new(),
        ));

        let mut graph: Graph<Package, ()> = Graph::new();
        let npm_package_node = graph.add_node(npm_package);
        let scoped_npm_package_node = graph.add_node(scoped_npm_package);
        let workspace_package_node = graph.add_node(workspace_package);

        graph.add_edge(workspace_package_node, npm_package_node, ());
        graph.add_edge(workspace_package_node, scoped_npm_package_node, ());

        (workspace_package_node, graph)
    }

    #[test]
    fn new_creates_store_folder() {
        let tmp_dir = TempDir::new("jm-writer").unwrap();

        let archiver = DefaultArchiver::new();
        let downloader = TarDownloader::new(&archiver);
        let _ = Writer::new(tmp_dir.as_ref(), &downloader).unwrap();

        let expected_path = tmp_dir.path().join("node_modules").join(".jm");

        assert!(expected_path.exists());
    }

    #[tokio::test]
    async fn fails_when_downloader_fails() {
        struct FailingDownloader {}

        #[async_trait]
        impl Downloader for FailingDownloader {
            async fn download_to(
                &self,
                _package: &NpmPackage,
                _path: &Path,
            ) -> Result<(), JmError> {
                Err(JmError::new(String::from("Failing downloader")))
            }
        }

        let tmp_dir = TempDir::new("jm-writer").unwrap();
        let downloader = FailingDownloader {};
        let writer = Writer::new(tmp_dir.as_ref(), &downloader).unwrap();

        let (starting_node, graph) = create_graph();

        let result = writer.write(vec![starting_node], &graph).await;

        assert_eq!(
            result,
            Err(JmError::new(String::from("Failing downloader")))
        );
    }

    #[tokio::test]
    async fn succeeds_for_scoped_and_non_scoped_packages() {
        struct DummyDownloader {}

        #[async_trait]
        impl Downloader for DummyDownloader {
            async fn download_to(&self, _package: &NpmPackage, path: &Path) -> Result<(), JmError> {
                fs::write(path.join("index.js"), "")?;

                Ok(())
            }
        }

        let tmp_dir = TempDir::new("jm-writer").unwrap();
        let downloader = DummyDownloader {};
        let writer = Writer::new(tmp_dir.as_ref(), &downloader).unwrap();

        let (starting_node, graph) = create_graph();

        let result = writer.write(vec![starting_node], &graph).await;

        let expected_package_path = tmp_dir
            .path()
            .join("node_modules")
            .join(".jm")
            .join("p1@1.0.0")
            .join("index.js");
        let expected_scoped_package_path = tmp_dir
            .path()
            .join("node_modules")
            .join(".jm")
            .join("@scope/p1@2.0.0")
            .join("index.js");

        assert_eq!(result, Ok(()));
        assert!(expected_package_path.exists());
        assert!(expected_scoped_package_path.exists());
    }
}
