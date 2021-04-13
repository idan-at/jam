pub mod errors;

use directories::ProjectDirs;
use errors::JmCacheError;
use jm_common::sanitize_package_name;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new(cache_group: String, cache_name: &str) -> Result<Cache, JmCacheError> {
        match ProjectDirs::from("com", "jm", &cache_group) {
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

    pub fn set(&self, key: &str, value: String) -> Result<PathBuf, JmCacheError> {
        let key_path = self.cache_dir.join(sanitize_package_name(key));
        let mut file = File::create(&key_path)?;

        copy(&mut value.as_bytes(), &mut file)?;

        // fs::write(key_path.clone(), value.as_ref())?;

        Ok(key_path)
    }
}
