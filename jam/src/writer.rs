use crate::downloader::Downloader;
use crate::errors::JamError;
use crate::store::Store;
use futures::StreamExt;
use jam_core::package::Package;
use log::debug;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Dfs;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::symlink;
use std::path::Path;

const CONCURRENCY: usize = 20;

pub struct Writer<'a> {
    store: &'a Store,
    downloader: &'a dyn Downloader,
}

impl<'a> Writer<'a> {
    pub fn new(store: &'a Store, downloader: &'a dyn Downloader) -> Writer<'a> {
        Writer { store, downloader }
    }

    // TODO: handle .bin scripts
    // TODO: handle native modules
    pub async fn write(
        &self,
        starting_nodes: Vec<NodeIndex>,
        graph: &Graph<Package, ()>,
    ) -> Result<(), JamError> {
        let mut futures = vec![];

        for node in starting_nodes {
            let mut dfs = Dfs::new(graph, node);
            while let Some(nx) = dfs.next(graph) {
                // TODO: We can skip already seen nodes
                let package = &graph[nx];
                let dependencies: Vec<&Package> = graph.neighbors(nx).map(|n| &graph[n]).collect();

                futures.push(self.write_package(package, dependencies));
            }
        }

        futures::stream::iter(futures.into_iter())
            .buffer_unordered(CONCURRENCY)
            .collect::<Vec<Result<_, _>>>()
            .await
            .into_iter()
            .collect::<Result<(), JamError>>()
    }

    async fn write_package(
        &self,
        package: &Package,
        dependencies: Vec<&Package>,
    ) -> Result<(), JamError> {
        match package {
            Package::NpmPackage(npm_package) => {
                let path = self.store.package_root_path_in_store(npm_package);
                let package_files_path = self.store.package_code_path_in_store(npm_package);

                if !package_files_path.exists() {
                    debug!("Downloading {} to directory {:?}", &npm_package.name, &path);

                    fs::create_dir_all(&package_files_path)?;
                    self.downloader
                        .download_to(&npm_package, &package_files_path)
                        .await?;

                    for dependency in dependencies {
                        self.create_link(&path, dependency)?;
                    }
                }
            }
            Package::WorkspacePackage(workspace_package) => {
                fs::create_dir_all(&workspace_package.base_path.join("node_modules"))?;
                for dependency in dependencies {
                    self.create_link(&workspace_package.base_path, dependency)?;
                }
            }
        }

        Ok(())
    }

    fn create_link(&self, package_root_path: &Path, to_package: &Package) -> Result<(), JamError> {
        let original = match to_package {
            Package::NpmPackage(npm_package) => self.store.package_code_path_in_store(&npm_package),
            Package::WorkspacePackage(workspace_package) => workspace_package.base_path.clone(),
        };

        let link = package_root_path
            .join("node_modules")
            .join(&to_package.name());

        // TODO: move to a linker component
        fs::create_dir_all(link.parent().unwrap())?;
        if let Err(err) = symlink(&original, &link) {
            if err.kind() != ErrorKind::AlreadyExists {
                return Err(JamError::new(format!(
                    "Failed to link package {:?}->{:?} {}",
                    link,
                    original,
                    err.to_string()
                )));
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
    use jam_cache::CacheFactory;
    use jam_core::package::NpmPackage;
    use jam_core::package::WorkspacePackage;
    use maplit::hashmap;
    use tempdir::TempDir;

    fn create_context() -> (
        Vec<NodeIndex>,
        Vec<WorkspacePackage>,
        Graph<Package, ()>,
        TempDir,
    ) {
        let tmp_dir = TempDir::new("jam-writer").unwrap();

        let npm_package = Package::NpmPackage(NpmPackage::new(
            "p1".to_string(),
            "1.0.0".to_string(),
            None,
            "shasum".to_string(),
            "tarball-url".to_string(),
            vec![],
        ));
        let scoped_npm_package = Package::NpmPackage(NpmPackage::new(
            "@scope/p1".to_string(),
            "2.0.0".to_string(),
            Some(hashmap! {
                "p1".to_string() => "1.0.0".to_string(),
            }),
            "shasum".to_string(),
            "tarball-url".to_string(),
            vec![],
        ));
        let workspace_package_inner = WorkspacePackage::new(
            "workspace_package".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
                "p1".to_string() => "1.0.0".to_string(),
                "@scope/p1".to_string() => "2.0.0".to_string(),
            }),
            None,
            tmp_dir.path().join("wp1"),
        );
        let workspace_package = Package::WorkspacePackage(workspace_package_inner.clone());
        let workspace_package2_inner = WorkspacePackage::new(
            "workspace_package2".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
                "p1".to_string() => "1.0.0".to_string(),
                "workspace_package".to_string() => "1.0.0".to_string(),
            }),
            None,
            tmp_dir.path().join("wp2"),
        );
        let workspace_package2 = Package::WorkspacePackage(workspace_package2_inner.clone());

        let mut graph: Graph<Package, ()> = Graph::new();
        let npm_package_node = graph.add_node(npm_package);
        let scoped_npm_package_node = graph.add_node(scoped_npm_package);
        let workspace_package_node = graph.add_node(workspace_package);
        let workspace_package2_node = graph.add_node(workspace_package2);

        graph.add_edge(workspace_package_node, npm_package_node, ());
        graph.add_edge(workspace_package_node, scoped_npm_package_node, ());
        graph.add_edge(scoped_npm_package_node, npm_package_node, ());
        graph.add_edge(workspace_package2_node, npm_package_node, ());
        graph.add_edge(workspace_package2_node, workspace_package_node, ());

        (
            vec![workspace_package_node, workspace_package2_node],
            vec![workspace_package_inner, workspace_package2_inner],
            graph,
            tmp_dir,
        )
    }

    #[test]
    fn new_creates_store_folder() {
        let tmp_dir = TempDir::new("jam-writer").unwrap();
        let cache_factory = CacheFactory::new(tmp_dir.path().join("cache_factory"));

        let archiver = DefaultArchiver::new();
        let downloader = TarDownloader::new(&cache_factory, &archiver).unwrap();
        let store = Store::new(tmp_dir.as_ref()).unwrap();
        let _ = Writer::new(&store, &downloader);

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
            ) -> Result<(), JamError> {
                Err(JamError::new(String::from("Failing downloader")))
            }
        }
        let downloader = FailingDownloader {};

        let (starting_nodes, _, graph, tmp_dir) = create_context();
        let store = Store::new(tmp_dir.as_ref()).unwrap();
        let writer = Writer::new(&store, &downloader);

        let result = writer.write(starting_nodes, &graph).await;

        assert_eq!(
            result,
            Err(JamError::new(String::from("Failing downloader")))
        );
    }

    #[tokio::test]
    async fn succeeds_for_scoped_and_non_scoped_packages() {
        struct DummyDownloader {}

        #[async_trait]
        impl Downloader for DummyDownloader {
            async fn download_to(
                &self,
                _package: &NpmPackage,
                path: &Path,
            ) -> Result<(), JamError> {
                fs::write(path.join("index.js"), "")?;

                Ok(())
            }
        }
        let downloader = DummyDownloader {};

        let (starting_nodes, workspace_packages, graph, tmp_dir) = create_context();
        let store = Store::new(tmp_dir.as_ref()).unwrap();
        let writer = Writer::new(&store, &downloader);

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

        let expected_scoped_package_to_package_link_path = fs::read_link(
            tmp_dir
                .path()
                .join("store")
                .join("@scope_p1@2.0.0")
                .join("node_modules")
                .join("p1"),
        )
        .unwrap();

        let expected_workspace_package_to_package_link_path = fs::read_link(
            workspace_packages[0]
                .base_path
                .clone()
                .join("node_modules")
                .join("p1"),
        )
        .unwrap();
        let expected_workspace_package_to_scoped_package_link_path = fs::read_link(
            workspace_packages[0]
                .base_path
                .clone()
                .join("node_modules")
                .join("@scope")
                .join("p1"),
        )
        .unwrap();

        let expected_workspace_package2_to_package_link_path = fs::read_link(
            workspace_packages[1]
                .base_path
                .clone()
                .join("node_modules")
                .join("p1"),
        )
        .unwrap();
        let expected_workspace_package2_to_workspace_package_link_path = fs::read_link(
            workspace_packages[1]
                .base_path
                .clone()
                .join("node_modules")
                .join("workspace_package"),
        )
        .unwrap();

        assert_eq!(result, Ok(()));

        assert!(expected_package_path.exists());
        assert!(expected_scoped_package_path.exists());

        assert_eq!(
            expected_scoped_package_to_package_link_path,
            expected_package_path.parent().unwrap()
        );

        assert_eq!(
            expected_workspace_package_to_package_link_path,
            expected_package_path.parent().unwrap()
        );
        assert_eq!(
            expected_workspace_package_to_scoped_package_link_path,
            expected_scoped_package_path.parent().unwrap()
        );

        assert_eq!(
            expected_workspace_package2_to_package_link_path,
            expected_package_path.parent().unwrap()
        );
        assert_eq!(
            expected_workspace_package2_to_workspace_package_link_path,
            workspace_packages[0].base_path
        );
    }
}
