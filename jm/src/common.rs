use std::fs;
use std::path::PathBuf;

pub fn read_manifest_file<'a>(manifest_file_path: PathBuf) -> Result<String, String> {
    if manifest_file_path.exists() {
        match fs::read_to_string(&manifest_file_path) {
            Ok(content) => Ok(content),
            Err(err) => Err(String::from(err.to_string())),
        }
    } else {
        Err(format!(
            "Couldn't find manifest file in {:?}",
            manifest_file_path
        ))
    }
}
