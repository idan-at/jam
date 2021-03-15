use crate::config::Config;
use jm_core::package::Package;
use petgraph::Graph;

use std::fs;
use std::path::PathBuf;

pub struct Writer {
    root_path: PathBuf,
}

impl Writer {
    pub fn new(config: &Config) -> Result<Writer, String> {
        let writer = Writer {
            root_path: config.root_path.clone(),
        };

        let hidden_path = writer.root_path.as_path().join("node_modules").join(".jm");
        match fs::create_dir_all(&hidden_path) {
            Ok(_) => Ok(writer),
            Err(err) => Err(String::from(err.to_string())),
        }
    }

    pub fn write(&self, _graph: &Graph<Package, ()>) -> Result<(), String> {
        Ok(())
    }
}
