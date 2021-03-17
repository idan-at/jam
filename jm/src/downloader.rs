use jm_core::package::NpmPackage;
use reqwest::Client;
use std::path::PathBuf;
use flate2::read::GzDecoder;
use jm_core::errors::JmError;
use std::fs::File;
use std::path::Path;
use tar::Archive;
use std::io::copy;
use directories::{ProjectDirs};

pub struct Downloader {
    client: Client,
    cache_dir: PathBuf,
}

// TODO: test
impl Downloader {
    pub fn new() -> Downloader {
        let project_dirs = ProjectDirs::from("com", "jm", "jm").unwrap();

        Downloader {
            client: Client::new(),
            cache_dir: project_dirs.cache_dir().to_path_buf()
        }
    }

    pub async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JmError> {
        let tarball_name = format!("{}@{}", package.name, package.version);

        let archive_path = self.cache_dir.join(tarball_name);
        let response = self.client.get(&package.tarball_url).send().await?;
        let mut target = File::create(&archive_path)?;
        let content =  response.text().await?;
        copy(&mut content.as_bytes(), &mut target)?;

        let tar_gz = File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(path)?;

        Ok(())
    }
}
