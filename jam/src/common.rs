use crate::errors::JamError;
use std::fs;
use std::path::PathBuf;

pub fn read_manifest_file<'a>(manifest_file_path: PathBuf) -> Result<String, JamError> {
    let content = fs::read_to_string(&manifest_file_path)?;

    Ok(content)
}
