use std::path::{Path,PathBuf};
use std::fs;
use std::error::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
  workspaces: Vec<String>
}

#[derive(Debug)]
struct Config {
  workspaces: Vec<String>
}

impl Config {
  fn new(workspaces: Vec<String>) -> Config {
    Config { workspaces }
  }
}

pub fn run(cwd: &PathBuf) -> Result<(), String> {
    let manifest_file_path = cwd.join("package.json");

    match get_config(manifest_file_path.as_path()) {
      Ok(config) => {
        println!("{:?}", config);
        Ok(())
      },
      Err(err) => Err(err)
    }
}

fn get_config(manifest_file_path: &Path)-> Result<Config, String> {
  if manifest_file_path.exists() {
    match fs::read_to_string(manifest_file_path) {
      Ok(content) => {
        match serde_json::from_str::<Manifest>(&content) {
          Ok(manifest) => Ok(Config::new(manifest.workspaces)),
          Err(_) => Err(format!("Fail to parse {:?}, please make sure it is a valid JSON and 'workspaces' array exists", manifest_file_path))

        }
      }
      Err(err) => Err(String::from(err.description()))
    }
  } else {
      Err(format!("Couldn't find manifest file in {:?}", manifest_file_path))
  }
}
