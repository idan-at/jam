pub mod errors;

use dashmap::DashMap;
use directories::ProjectDirs;
use errors::JmCacheError;
use std::fs;
use std::path::PathBuf;

pub struct Cache {
    cache: DashMap<String, String>,
    cache_dir: PathBuf,
}

pub type CacheValue = (String, Option<PathBuf>);

// TODO: Test
impl Cache {
    pub fn new(cache_name: &str) -> Result<Cache, JmCacheError> {
        let cache: DashMap<String, String> = DashMap::new();

        match ProjectDirs::from("com", "jm", "jm") {
            Some(project_dirs) => {
                let cache_dir = project_dirs.cache_dir().to_path_buf().join(cache_name);

                fs::create_dir_all(&cache_dir)?;
                Ok(Cache { cache, cache_dir })
            }
            None => Err(JmCacheError::new(String::from(
                "Failed to create a cache directory",
            ))),
        }
    }

    pub fn get(&self, key: &str) -> Option<CacheValue> {
        if let Some(value) = self.get_from_memory_cache(key) {
            Some(value)
        } else {
            self.get_from_file_system_cache(key)
        }
    }

    pub fn set(&self, key: &str, value: String) {
        let key_path = self.cache_dir.join(key);

        // Ignoring if write succeeded or not
        #[allow(unused_must_use)]
        {
            fs::write(key_path, value.clone());
        }
        self.cache.insert(key.to_string(), value);
    }

    fn get_from_file_system_cache(&self, key: &str) -> Option<CacheValue> {
        let key_path = self.cache_dir.join(key);

        if key_path.exists() {
            match fs::read_to_string(&key_path) {
                Ok(value) => Some((value, Some(key_path))),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    fn get_from_memory_cache(&self, key: &str) -> Option<CacheValue> {
        match self.cache.get(key) {
            Some(value) => Some((value.clone(), None)),
            None => None,
        }
    }
}
