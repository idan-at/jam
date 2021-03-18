use async_trait::async_trait;
use directories::ProjectDirs;
use flate2::read::GzDecoder;
use jm_core::errors::JmError;
use jm_core::package::NpmPackage;
use log::debug;
use reqwest::Client;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use tar::Archive;

const NPM_PACK_PATH_PREFIX: &'static str = "package";

#[async_trait]
pub trait Downloader {
    async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JmError>;
}

pub struct TarDownloader {
    client: Client,
    cache_dir: PathBuf,
}

impl TarDownloader {
    pub fn new() -> TarDownloader {
        let project_dirs = ProjectDirs::from("com", "jm", "jm").unwrap();

        TarDownloader {
            client: Client::new(),
            cache_dir: project_dirs.cache_dir().to_path_buf(),
        }
    }

    async fn download_tar(&self, package: &NpmPackage) -> Result<PathBuf, JmError> {
        let now = Instant::now();

        let tarball_name = format!("{}@{}", package.name.replace("/", "_"), package.version);
        let archive_path = self.cache_dir.join(tarball_name);

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

// TODO: test
#[async_trait]
impl Downloader for TarDownloader {
    async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JmError> {
        debug!("Downloading tar of {} to {:?}", package.name, path);
        let now = Instant::now();
        let archive_path = self.download_tar(package).await?;

        let tar_gz = File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        for mut entry in archive.entries()?.filter_map(|e| e.ok()) {
            let entry_path = entry.path()?;
            let file_inner_path = match entry_path.strip_prefix(NPM_PACK_PATH_PREFIX) {
                Ok(stripped_path) => stripped_path.to_owned(),
                Err(_) => entry_path.to_path_buf(),
            };

            let file_path = path.join(&file_inner_path);
            fs::create_dir_all(&file_path.parent().unwrap())?;

            entry.unpack(file_path)?;
        }

        debug!(
            "Successfully extracted {} package tar in {} milliseconds",
            package.name,
            now.elapsed().as_millis()
        );

        Ok(())
    }
}
