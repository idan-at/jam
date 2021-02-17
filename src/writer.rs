use crate::config::Config;

use std::fs;
use std::path::PathBuf;

pub struct Writer {
    root_path: PathBuf,
}

impl Writer {
    pub fn new(config: &Config) -> Writer {
        Writer {
            root_path: config.root_path.clone(),
        }
    }

    // TODO: error handling
    pub fn init(&self) {
        let hidden_path = self.root_path.as_path().join("node_modules").join(".jm");
        fs::create_dir_all(&hidden_path).unwrap();
    }
}
