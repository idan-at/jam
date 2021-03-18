use flate2::read::GzDecoder;
use jm_core::errors::JmError;
use std::fs;
use std::fs::File;
use std::path::Path;
use tar::Archive;

const NPM_PACK_PATH_PREFIX: &'static str = "package";

pub trait Archiver: Send + Sync {
    fn extract_to(&self, tarball_path: &Path, target_path: &Path) -> Result<(), JmError>;
}

#[derive(Debug, Clone)]
pub struct DefaultArchiver {}

impl DefaultArchiver {
    pub fn new() -> DefaultArchiver {
        DefaultArchiver {}
    }
}

impl Archiver for DefaultArchiver {
    fn extract_to(&self, archive_path: &Path, target_path: &Path) -> Result<(), JmError> {
        let tar_gz = File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        for mut entry in archive.entries()?.filter_map(|e| e.ok()) {
            let entry_path = entry.path()?;
            let file_inner_path = match entry_path.strip_prefix(NPM_PACK_PATH_PREFIX) {
                Ok(stripped_path) => stripped_path.to_owned(),
                Err(_) => entry_path.to_path_buf(),
            };

            let file_path = target_path.join(&file_inner_path);
            fs::create_dir_all(&file_path.parent().unwrap())?;

            entry.unpack(file_path)?;
        }

        Ok(())
    }
}
