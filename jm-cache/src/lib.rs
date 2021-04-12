pub mod errors;

use directories::ProjectDirs;
use errors::JmCacheError;
use std::fs;
use std::path::PathBuf;

pub struct Cache {
    cache_dir: PathBuf,
}

pub type CacheValue = (Vec<u8>, Option<PathBuf>);

// TODO: unit tests
// TODO: expose a trait so it can be overriden in tests
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

    pub fn get(&self, key: &str) -> Option<CacheValue> {
        let key_path = self.cache_dir.join(key);

        if key_path.exists() {
            match fs::read(&key_path) {
                Ok(value) => Some((value, Some(key_path))),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn set(&self, key: &str, value: &[u8]) {
        let key_path = self.cache_dir.join(key);

        // Ignoring if write succeeded or not
        #[allow(unused_must_use)]
        {
            fs::write(key_path, value);
        }
    }
}
