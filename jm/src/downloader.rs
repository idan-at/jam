use crate::archiver::Archiver;
use crate::errors::JmError;
use async_trait::async_trait;
use jm_cache::Cache;
use jm_core::package::NpmPackage;
use log::{debug, info};
use reqwest::Client;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

#[async_trait]
pub trait Downloader {
    async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JmError>;
}

pub struct TarDownloader<'a> {
    client: Client,
    cache: Cache,
    archiver: &'a dyn Archiver,
}

impl<'a> TarDownloader<'a> {
    pub fn new(cache_group: String, archiver: &'a dyn Archiver) -> Result<TarDownloader, JmError> {
        let cache = Cache::new(cache_group, "tarballs")?;

        Ok(TarDownloader {
            client: Client::new(),
            cache,
            archiver,
        })
    }

    async fn download_tar(
        &self,
        package: &NpmPackage,
        tarball_name: &str,
    ) -> Result<PathBuf, JmError> {
        let response = self.client.get(&package.tarball_url).send().await?;
        let content = response.bytes().await?;

        let archive_path = self.cache.set(tarball_name, &content)?;

        Ok(archive_path)
    }
}

#[async_trait]
impl<'a> Downloader for TarDownloader<'a> {
    async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JmError> {
        let tarball_name = format!("{}@{}", package.name, package.version);

        let archive_path = match self.cache.get(&tarball_name) {
            Some(file_path) => {
                debug!("tar of {} found in cache", package.name);
                file_path
            }
            None => {
                let now = Instant::now();
                let archive_path = self.download_tar(package, &tarball_name).await?;

                debug!(
                    "Downloaded {} package tar in {} milliseconds",
                    package.name,
                    now.elapsed().as_millis()
                );

                archive_path
            }
        };

        info!("Extracting {} to {:?}", package.name, path);
        self.archiver.extract_to(&archive_path, path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jm_test_utils::npm_mock_server::*;
    use maplit::hashmap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tempdir::TempDir;

    fn setup() -> NpmMockServer {
        let npm_mock_server = NpmMockServer::new();

        npm_mock_server
    }

    #[tokio::test]
    async fn fails_when_archiver_fails() {
        let mut npm_mock_server = setup();

        struct FailingArchiver {}

        impl Archiver for FailingArchiver {
            fn extract_to(&self, _tarball_path: &Path, _target_path: &Path) -> Result<(), JmError> {
                Err(JmError::new(String::from("Failing archiver")))
            }
        }

        let package = NpmPackage::new(
            "p1".to_string(),
            "1.0.0".to_string(),
            None,
            "shasum".to_string(),
            format!("{}/tarball/{}", npm_mock_server.url(), "p1"),
        );
        let path = PathBuf::new();

        let archiver = FailingArchiver {};
        let downloader = TarDownloader::new("tests".to_string(), &archiver).unwrap();

        npm_mock_server.with_tarball_data(
            "p1",
            hashmap! { "index.js".to_string() => "const x = 1".to_string() },
        );

        let result = downloader.download_to(&package, path.as_path()).await;

        assert_eq!(result, Err(JmError::new(String::from("Failing archiver"))));
    }

    #[tokio::test]
    async fn calls_the_archiver_with_the_tar_path_and_target_path() {
        let mut npm_mock_server = setup();

        struct MockArchiver {
            pub called_with: Arc<Mutex<Vec<PathBuf>>>,
        }

        impl MockArchiver {
            pub fn new() -> MockArchiver {
                MockArchiver {
                    called_with: Arc::new(Mutex::new(vec![])),
                }
            }
        }

        impl Archiver for MockArchiver {
            fn extract_to(&self, _: &Path, target_path: &Path) -> Result<(), JmError> {
                let mut lock = self.called_with.lock().unwrap();

                (*lock).push(target_path.to_path_buf());

                Ok(())
            }
        }

        let package = NpmPackage::new(
            "p1".to_string(),
            "1.0.0".to_string(),
            None,
            "shasum".to_string(),
            format!("{}/tarball/{}", npm_mock_server.url(), "p1"),
        );
        let scoped_package = NpmPackage::new(
            "@scoped/p1".to_string(),
            "2.0.0".to_string(),
            None,
            "shasum".to_string(),
            format!("{}/tarball/{}", npm_mock_server.url(), "%40scoped%2Fp2"),
        );

        let tmp_dir = TempDir::new("jm-downloader").unwrap();

        let archiver = MockArchiver::new();
        let downloader = TarDownloader::new("tests".to_string(), &archiver).unwrap();

        npm_mock_server.with_tarball_data(
            "p1",
            hashmap! { "index.js".to_string() => "const x = 1".to_string() },
        );
        npm_mock_server.with_tarball_data(
            "@scoped/p2",
            hashmap! { "index.js".to_string() => "const x = 2".to_string() },
        );

        downloader
            .download_to(&package, tmp_dir.path().join("p1").as_path())
            .await
            .unwrap();
        downloader
            .download_to(&scoped_package, tmp_dir.path().join("@scoped_p2").as_path())
            .await
            .unwrap();

        let expected_paths = vec![
            tmp_dir.path().to_path_buf().join("p1"),
            tmp_dir.path().to_path_buf().join("@scoped_p2"),
        ];

        let called_with = archiver.called_with.lock().unwrap();

        assert_eq!(*called_with, expected_paths);
    }
}
