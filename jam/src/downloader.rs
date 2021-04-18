use crate::archiver::Archiver;
use crate::errors::JamError;
use async_trait::async_trait;
use jam_cache::{Cache, CacheFactory};
use jam_core::package::NpmPackage;
use log::{debug, info};
use reqwest::Client;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

#[async_trait]
pub trait Downloader {
    async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JamError>;
}

pub struct TarDownloader<'a> {
    client: Client,
    cache: Cache,
    archiver: &'a dyn Archiver,
}

impl<'a> TarDownloader<'a> {
    pub fn new(
        cache_factory: &CacheFactory,
        archiver: &'a dyn Archiver,
    ) -> Result<TarDownloader<'a>, JamError> {
        let cache = cache_factory.create_cache("tarballs")?;

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
    ) -> Result<PathBuf, JamError> {
        let response = self.client.get(&package.tarball_url).send().await?;
        let content = response.bytes().await?;

        let archive_path = self.cache.set(tarball_name, &content)?;

        Ok(archive_path)
    }
}

#[async_trait]
impl<'a> Downloader for TarDownloader<'a> {
    async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JamError> {
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
    use jam_test_utils::npm_mock_server::*;
    use maplit::hashmap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tempdir::TempDir;

    fn setup() -> (NpmMockServer, TempDir, CacheFactory) {
        let npm_mock_server = NpmMockServer::new();
        let tmp_dir = TempDir::new("jam-downloader").unwrap();

        let cache_factory = CacheFactory::new(tmp_dir.path().to_path_buf());

        (npm_mock_server, tmp_dir, cache_factory)
    }

    #[tokio::test]
    async fn fails_when_archiver_fails() {
        let (mut npm_mock_server, _, cache_factory) = setup();

        struct FailingArchiver {}

        impl Archiver for FailingArchiver {
            fn extract_to(&self, _tarball_path: &Path, _target_path: &Path) -> Result<(), JamError> {
                Err(JamError::new(String::from("Failing archiver")))
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
        let downloader = TarDownloader::new(&cache_factory, &archiver).unwrap();

        npm_mock_server.with_tarball_data(
            "p1",
            hashmap! { "index.js".to_string() => "const x = 1".to_string() },
        );

        let result = downloader.download_to(&package, path.as_path()).await;

        assert_eq!(result, Err(JamError::new(String::from("Failing archiver"))));
    }

    #[tokio::test]
    async fn calls_the_archiver_with_the_tar_path_and_target_path() {
        let (mut npm_mock_server, tmp_dir, cache_factory) = setup();

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
            fn extract_to(&self, _: &Path, target_path: &Path) -> Result<(), JamError> {
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

        let archiver = MockArchiver::new();
        let downloader = TarDownloader::new(&cache_factory, &archiver).unwrap();

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
