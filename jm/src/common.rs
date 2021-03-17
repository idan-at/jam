use jm_core::errors::JmError;
use std::fs;
use std::path::PathBuf;

pub fn read_manifest_file<'a>(manifest_file_path: PathBuf) -> Result<String, JmError> {
    let content = fs::read_to_string(&manifest_file_path)?;

    Ok(content)
}
