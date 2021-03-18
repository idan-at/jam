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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
use tempdir::TempDir;
    use super::*;
    use std::env;

    fn context() -> PathBuf {
        env::current_dir().unwrap().join("tests").join("fixtures")
    }

    #[test]
    fn extracts_archive_to_given_path_while_ignoring_the_pack_prefix() {
        let archiver = DefaultArchiver::new();
        let tmp_dir = TempDir::new("jm-archiver").unwrap();

        let fixtures_path = context();
        let archive_path = fixtures_path.join("simple.tgz");

        let expected_file_path = tmp_dir.path().join("package.json");

        let result = archiver.extract_to(&archive_path, tmp_dir.path());

        assert!(result.is_ok());
        assert!(expected_file_path.exists());
    }

    #[test]
    fn extracts_archive_to_given_path_even_if_pack_prefix_does_not_exist() {
        let archiver = DefaultArchiver::new();
        let tmp_dir = TempDir::new("jm-archiver").unwrap();

        let fixtures_path = context();
        let archive_path = fixtures_path.join("no_pack_prefix.tgz");

        let expected_file_path = tmp_dir.path().join("package.json");

        let result = archiver.extract_to(&archive_path, tmp_dir.path());

        assert!(result.is_ok());
        assert!(expected_file_path.exists());
    }

    #[test]
    fn does_not_fail_on_archives_packed_with_root_permissions() {
        let archiver = DefaultArchiver::new();
        let tmp_dir = TempDir::new("jm-archiver").unwrap();

        let fixtures_path = context();
        let archive_path = fixtures_path.join("root_permissions.tgz");

        let expected_file_path = tmp_dir.path().join("package.json");

        let result = archiver.extract_to(&archive_path, tmp_dir.path());

        assert!(result.is_ok());
        assert!(expected_file_path.exists());
    }
}