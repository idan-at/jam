use crate::downloader::Downloader;
use crate::errors::JmError;
use jm_common::sanitize_package_name;
use jm_core::package::NpmPackage;
use jm_core::package::Package;
use log::debug;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Dfs;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;

pub struct Writer<'a> {
    store_path: PathBuf,
    downloader: &'a dyn Downloader,
}

impl<'a> Writer<'a> {
    pub fn new(data_dir: &Path, downloader: &'a dyn Downloader) -> Result<Writer<'a>, JmError> {
        let store_path = data_dir.join("store");

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
                let neighbors: Vec<&Package> = graph.neighbors(nx).map(|n| &graph[n]).collect();
                self.write_package(package, neighbors).await?;
            }
        }

        Ok(())
    }

    async fn write_package(
        &self,
        package: &Package,
        neighbors: Vec<&Package>,
    ) -> Result<(), JmError> {
        match package {
            Package::NpmPackage(npm_package) => {
                let path = self.package_store_root_path(npm_package);
                let package_files_path = self.package_code_path(npm_package);

                if !package_files_path.exists() {
                    debug!("Downloading {} to directory {:?}", &npm_package.name, &path);

                    fs::create_dir_all(&package_files_path)?;
                    self.downloader
                        .download_to(&npm_package, &package_files_path)
                        .await?;

                    for neighbor in neighbors {
                        self.create_link(path.clone(), neighbor)?;
                    }
                }
            }
            Package::WorkspacePackage(workspace_package) => {
                debug!("Ignoring workspace package {:?}", workspace_package);

                fs::create_dir_all(&workspace_package.base_path.join("node_modules"))?;
            }
        }

        Ok(())
    }

    fn package_store_root_path(&self, package: &NpmPackage) -> PathBuf {
        let package_dir_name = format!(
            "{}@{}",
            sanitize_package_name(&package.name),
            package.version
        );

        self.store_path.join(package_dir_name)
    }

    fn package_code_path(&self, package: &NpmPackage) -> PathBuf {
        self.package_store_root_path(package)
            .join("node_modules")
            .join(&package.name)
    }

    fn create_link(&self, package_root_path: PathBuf, to_package: &Package) -> Result<(), JmError> {
        match to_package {
            Package::NpmPackage(npm_package) => {
                let original = self.package_code_path(&npm_package);
                let link = package_root_path
                    .join("node_modules")
                    .join(&npm_package.name);

                println!("parent exists {}", link.parent().unwrap().exists());

                if let Err(err) = symlink(&original, &link) {
                    if err.kind() != ErrorKind::AlreadyExists {
                        return Err(JmError::new(format!(
                            "Failed to link package {:?}->{:?} {}",
                            link,
                            original,
                            err.to_string()
                        )));
                    }
                }
            }
            Package::WorkspacePackage(workspace_package) => {
                debug!("Not linking workspace package {:?}", workspace_package)
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archiver::DefaultArchiver;
    use crate::downloader::TarDownloader;
    use async_trait::async_trait;
    use jm_cache::CacheFactory;
    use jm_core::package::WorkspacePackage;
    use maplit::hashmap;
    use tempdir::TempDir;

    fn create_graph() -> (Vec<NodeIndex>, Graph<Package, ()>) {
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
        let workspace_package2 = Package::WorkspacePackage(WorkspacePackage::new(
            "workspace_package2".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
                "p1".to_string() => "1.0.0".to_string(),
            }),
            None,
            PathBuf::new(),
        ));

        let mut graph: Graph<Package, ()> = Graph::new();
        let npm_package_node = graph.add_node(npm_package);
        let scoped_npm_package_node = graph.add_node(scoped_npm_package);
        let workspace_package_node = graph.add_node(workspace_package);
        let workspace_package2_node = graph.add_node(workspace_package2);

        graph.add_edge(workspace_package_node, npm_package_node, ());
        graph.add_edge(workspace_package_node, scoped_npm_package_node, ());
        graph.add_edge(workspace_package2_node, npm_package_node, ());

        (vec![workspace_package_node, workspace_package2_node], graph)
    }

    #[test]
    fn new_creates_store_folder() {
        let tmp_dir = TempDir::new("jm-writer").unwrap();
        let cache_factory = CacheFactory::new(tmp_dir.path().join("cache_factory"));

        let archiver = DefaultArchiver::new();
        let downloader = TarDownloader::new(&cache_factory, &archiver).unwrap();
        let _ = Writer::new(tmp_dir.as_ref(), &downloader).unwrap();

        let expected_path = tmp_dir.path().join("store");

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

        let (starting_nodes, graph) = create_graph();

        let result = writer.write(starting_nodes, &graph).await;

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

        let (starting_nodes, graph) = create_graph();

        let result = writer.write(starting_nodes, &graph).await;

        let expected_package_path = tmp_dir
            .path()
            .join("store")
            .join("p1@1.0.0")
            .join("node_modules")
            .join("p1")
            .join("index.js");
        let expected_scoped_package_path = tmp_dir
            .path()
            .join("store")
            .join("@scope_p1@2.0.0")
            .join("node_modules")
            .join("@scope")
            .join("p1")
            .join("index.js");

        // TODO: test created links
        assert_eq!(result, Ok(()));
        assert!(expected_package_path.exists());
        assert!(expected_scoped_package_path.exists());
    }
}
