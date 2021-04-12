pub mod errors;

use jm_common::sanitize_package_name;
use directories::ProjectDirs;
use errors::JmCacheError;
use std::fs;
use std::path::PathBuf;

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new(cache_name: &str) -> Result<Cache, JmCacheError> {
        match ProjectDirs::from("com", "jm", "jm") {
            Some(project_dirs) => {
                let cache_dir = project_dirs.cache_dir().to_path_buf().join(cache_name);

                fs::create_dir_all(&cache_dir)?;
                Ok(Cache { cache_dir })
            }
            None => Err(JmCacheError::new(String::from(
                "Failed to find/create a cache directory",
            ))),
        }
    }

    pub fn get(&self, package_name: &str) -> Option<PathBuf> {
        let key_path = self.cache_dir.join(sanitize_package_name(package_name));

        if key_path.exists() {
            Some(key_path)
        } else {
            None
        }
    }

    pub fn set(&self, package_name: &str, value: String) -> Result<PathBuf, JmCacheError> {
        let key_path = self.cache_dir.join(sanitize_package_name(package_name));

        fs::write(key_path.clone(), value)?;

        Ok(key_path)
    }
}
