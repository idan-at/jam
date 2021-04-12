pub mod errors;

use directories::ProjectDirs;
use errors::JmCacheError;
use jm_common::sanitize_package_name;
use std::fs;
use std::path::PathBuf;

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new(cache_group: String, cache_name: &str) -> Result<Cache, JmCacheError> {
        match ProjectDirs::from("com", &cache_group, "jm") {
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

    pub fn get(&self, key: &str) -> Option<PathBuf> {
        let key_path = self.cache_dir.join(sanitize_package_name(key));

        if key_path.exists() {
            Some(key_path)
        } else {
            None
        }
    }

    pub fn set<C: AsRef<[u8]>>(&self, key: &str, value: C) -> Result<PathBuf, JmCacheError> {
        let key_path = self.cache_dir.join(sanitize_package_name(key));

        fs::write(key_path.clone(), value.as_ref())?;

        Ok(key_path)
    }
}
