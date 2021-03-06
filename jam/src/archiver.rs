use crate::errors::JamError;
use flate2::read::GzDecoder;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use tar::{Archive, EntryType};

// Default npm pack directory
const NPM_PACK_PATH_PREFIX: &'static str = "package";
// @types/node pack directory
const TYPES_NODE_PATH_PREFIX: &'static str = "node";

pub trait Archiver: Send + Sync {
    fn extract_to(&self, tarball_path: &Path, target_path: &Path) -> Result<(), JamError>;
}

#[derive(Debug, Clone)]
pub struct DefaultArchiver {}

impl DefaultArchiver {
    pub fn new() -> DefaultArchiver {
        DefaultArchiver {}
    }

    fn strip_known_prefixes(&self, path: &Path) -> PathBuf {
        let sanitized_path = match path.strip_prefix(NPM_PACK_PATH_PREFIX) {
            Ok(stripped_path) => stripped_path.to_owned(),
            Err(_) => path.to_path_buf(),
        };

        match sanitized_path.strip_prefix(TYPES_NODE_PATH_PREFIX) {
            Ok(final_path) => final_path.to_owned(),
            Err(_) => sanitized_path,
        }
    }
}

impl Archiver for DefaultArchiver {
    fn extract_to(&self, archive_path: &Path, target_path: &Path) -> Result<(), JamError> {
        let tar_gz = File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        for mut entry in archive.entries()?.filter_map(|e| e.ok()) {
            if entry.header().entry_type() == EntryType::Directory {
                continue;
            }

            let entry_path = entry.path()?;
            let file_inner_path = self.strip_known_prefixes(&entry_path);

            let file_path = target_path.join(&file_inner_path);
            fs::create_dir_all(&file_path.parent().unwrap())?;

            entry.unpack(file_path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;
    use tempdir::TempDir;

    fn get_fixtures_path() -> PathBuf {
        env::current_dir().unwrap().join("tests").join("fixtures")
    }

    #[test]
    fn extracts_archive_to_given_path_while_ignoring_the_pack_prefix() {
        let archiver = DefaultArchiver::new();
        let tmp_dir = TempDir::new("jam-archiver").unwrap();

        let fixtures_path = get_fixtures_path();
        let archive_path = fixtures_path.join("simple.tgz");

        let expected_file_path = tmp_dir.path().join("package.json");

        let result = archiver.extract_to(&archive_path, tmp_dir.path());

        assert!(result.is_ok());
        assert!(expected_file_path.exists());
    }

    #[test]
    fn extracts_archive_to_given_path_while_ignoring_a_unique_path_prefix() {
        let archiver = DefaultArchiver::new();
        let tmp_dir = TempDir::new("jam-archiver").unwrap();

        let fixtures_path = get_fixtures_path();
        let archive_path = fixtures_path.join("different_path_prefix.tgz");

        let expected_file_path = tmp_dir.path().join("package.json");

        let result = archiver.extract_to(&archive_path, tmp_dir.path());

        assert!(result.is_ok());
        assert!(expected_file_path.exists());
    }

    #[test]
    fn extracts_archive_to_given_path_even_if_pack_prefix_does_not_exist() {
        let archiver = DefaultArchiver::new();
        let tmp_dir = TempDir::new("jam-archiver").unwrap();

        let fixtures_path = get_fixtures_path();
        let archive_path = fixtures_path.join("no_pack_prefix.tgz");

        let expected_file_path = tmp_dir.path().join("package.json");

        let result = archiver.extract_to(&archive_path, tmp_dir.path());

        assert!(result.is_ok());
        assert!(expected_file_path.exists());
    }

    #[test]
    fn does_not_fail_on_archives_packed_with_different_permissions() {
        let archiver = DefaultArchiver::new();
        let tmp_dir = TempDir::new("jam-archiver").unwrap();

        let fixtures_path = get_fixtures_path();
        let archive_path = fixtures_path.join("different_permissions.tgz");

        let expected_file_path = tmp_dir.path().join("package.json");

        let result = archiver.extract_to(&archive_path, tmp_dir.path());

        assert!(result.is_ok());
        assert!(expected_file_path.exists());
    }

    #[test]
    fn overrides_existing_files() {
        let archiver = DefaultArchiver::new();
        let tmp_dir = TempDir::new("jam-archiver").unwrap();

        let fixtures_path = get_fixtures_path();
        let archive_path = fixtures_path.join("simple.tgz");

        let expected_file_path = tmp_dir.path().join("package.json");
        fs::write(&expected_file_path, "{}").unwrap();

        let result = archiver.extract_to(&archive_path, tmp_dir.path());

        assert!(result.is_ok());
        assert!(expected_file_path.exists());
        assert_ne!(fs::read_to_string(expected_file_path).unwrap(), "{}");
    }
}
