use crate::archiver::Archiver;
use async_trait::async_trait;
use directories::ProjectDirs;
use jm_core::errors::JmError;
use jm_core::package::NpmPackage;
use log::{debug, info};
use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

#[async_trait]
pub trait Downloader {
    async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JmError>;
}

pub struct TarDownloader<'a> {
    client: Client,
    cache_dir: PathBuf,
    archiver: &'a dyn Archiver,
}

impl<'a> TarDownloader<'a> {
    pub fn new(archiver: &'a dyn Archiver) -> TarDownloader {
        let project_dirs = ProjectDirs::from("com", "jm", "jm").unwrap();

        TarDownloader {
            client: Client::new(),
            cache_dir: project_dirs.cache_dir().to_path_buf(),
            archiver,
        }
    }

    async fn download_tar(&self, package: &NpmPackage) -> Result<PathBuf, JmError> {
        let now = Instant::now();

        let tarball_name = format!("{}@{}", package.name.replace("/", "_"), package.version);
        let archive_path = self.cache_dir.join(tarball_name);

        // TODO: add retries
        let response = self.client.get(&package.tarball_url).send().await?;
        let mut target = File::create(&archive_path)?;
        let content = response.bytes().await?;

        target.write_all(content.as_ref())?;
        debug!(
            "Downloaded {} package tar in {} milliseconds",
            package.name,
            now.elapsed().as_millis()
        );

        Ok(archive_path)
    }
}

#[async_trait]
impl<'a> Downloader for TarDownloader<'a> {
    async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JmError> {
        debug!("Downloading tar of {} to {:?}", package.name, path);
        let now = Instant::now();
        // TODO: check if exists first
        let archive_path = self.download_tar(package).await?;

        info!("Extracting {} to {:?}", package.name, path);
        self.archiver.extract_to(&archive_path, path)?;

        debug!(
            "Successfully extracted {} package tar in {} milliseconds",
            package.name,
            now.elapsed().as_millis()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempdir::TempDir;
    use maplit::hashmap;
    use jm_test_utils::npm_mock_server::*;

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
        let downloader = TarDownloader::new(&archiver);

        let package = NpmPackage::new(
            "p1".to_string(),
            "1.0.0".to_string(),
            None,
            "shasum".to_string(),
            format!("{}/tarball/{}", npm_mock_server.url(), "p1"),
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
            fn extract_to(&self, tarball_path: &Path, target_path: &Path) -> Result<(), JmError> {
                let mut lock = self.called_with.lock().unwrap();

                (*lock).push(tarball_path.to_path_buf());
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
        let tarballs_dir = ProjectDirs::from("com", "jm", "jm")
            .unwrap()
            .cache_dir()
            .to_path_buf();

        let tmp_dir = TempDir::new("jm-downloader").unwrap();

        let archiver = MockArchiver::new();
        let downloader = TarDownloader::new(&archiver);

        npm_mock_server.with_tarball_data(
            "p1",
            hashmap! { "index.js".to_string() => "const x = 1".to_string() },
        );
        npm_mock_server.with_tarball_data(
            "@scoped/p2",
            hashmap! { "index.js".to_string() => "const x = 2".to_string() },
        );

        downloader
            .download_to(&package, tmp_dir.path())
            .await
            .unwrap();
        downloader
            .download_to(&scoped_package, tmp_dir.path())
            .await
            .unwrap();

        let expected_paths = vec![
            tarballs_dir.join("p1@1.0.0"),
            tmp_dir.path().to_path_buf(),
            tarballs_dir.join("@scoped_p1@2.0.0"),
            tmp_dir.path().to_path_buf(),
        ];

        let called_with = archiver.called_with.lock().unwrap();

        assert_eq!(*called_with, expected_paths);
    }
}
