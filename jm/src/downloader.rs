use cached_path::cached_path;
use flate2::read::GzDecoder;
use jm_core::errors::JmError;
use std::fs::File;
use std::path::Path;
use tar::Archive;

pub struct Downloader {}

// TODO: test
impl Downloader {
    pub fn new() -> Downloader {
        Downloader {}
    }

    pub fn download_to(&self, tarball_url: &str, path: &Path) -> Result<(), JmError> {
        let archive_path = cached_path(tarball_url).unwrap();

        let tar_gz = File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(path)?;

        Ok(())
    }
}
