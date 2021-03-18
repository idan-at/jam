use directories::ProjectDirs;
use flate2::read::GzDecoder;
use jm_core::errors::JmError;
use jm_core::package::NpmPackage;
use reqwest::Client;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tar::Archive;

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
            cache_dir: project_dirs.cache_dir().to_path_buf(),
        }
    }

    pub async fn download_to(&self, package: &NpmPackage, path: &Path) -> Result<(), JmError> {
        let archive_path = self.download_tar(package).await?;

        let tar_gz = File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(path)?;

        Ok(())
    }

    async fn download_tar(&self, package: &NpmPackage) -> Result<PathBuf, JmError> {
        let tarball_name = format!("{}@{}", package.name, package.version);

        let archive_path = self.cache_dir.join(tarball_name);

        fs::create_dir_all(&archive_path.parent().unwrap())?;

        let response = self.client.get(&package.tarball_url).send().await?;
        let mut target = File::create(&archive_path)?;
        let content = response.bytes().await?;

        target.write_all(content.as_ref())?;

        Ok(archive_path)
    }
}
