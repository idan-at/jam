pub mod errors;

use errors::JmCacheError;
use jm_common::sanitize_package_name;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub struct CacheFactory {
    cache_dir: PathBuf,
}

pub struct Cache {
    cache_dir: PathBuf,
}

impl CacheFactory {
    pub fn new(cache_dir: PathBuf) -> CacheFactory {
        CacheFactory { cache_dir }
    }

    pub fn create_cache(&self, cache_name: &str) -> Result<Cache, JmCacheError> {
        let cache_dir = self.cache_dir.join(cache_name);
        fs::create_dir_all(&cache_dir)?;

        Ok(Cache { cache_dir })
    }
}

impl Cache {
    pub fn get(&self, key: &str) -> Option<PathBuf> {
        let key_path = self.cache_dir.join(sanitize_package_name(key));

        if key_path.exists() {
            Some(key_path)
        } else {
            None
        }
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<PathBuf, JmCacheError> {
        let key_path = self.cache_dir.join(sanitize_package_name(key));
        let mut file = File::create(&key_path)?;

        file.write_all(value)?;

        Ok(key_path)
    }
}
