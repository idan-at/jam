mod config;
mod workspace;

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use config::Config;
use workspace::Workspace;

pub fn run(root_path: PathBuf) -> Result<(), String> {
    let manifest_file_path = root_path.join("package.json");
    let manifest_file_content = read_manifest_file(manifest_file_path)?;

    let config = Config::new(root_path, &manifest_file_content)?;
    println!("{:?}", config);

    let workspace = Workspace::from_config(&config)?;

    println!("{:?}", workspace.packages);

    Ok(())
}

fn read_manifest_file<'a>(manifest_file_path: PathBuf) -> Result<String, String> {
    if manifest_file_path.exists() {
        match fs::read_to_string(&manifest_file_path) {
            Ok(content) => Ok(content),
            Err(err) => Err(String::from(err.description())),
        }
    } else {
        Err(format!(
            "Couldn't find manifest file in {:?}",
            manifest_file_path
        ))
    }
}
