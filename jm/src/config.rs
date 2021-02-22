use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Manifest {
    workspaces: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub root_path: PathBuf,
    pub patterns: Vec<String>,
    pub registry: String,
}

impl Config {
    pub fn new(root_path: PathBuf, manifest_file_content: &str, registry: &str) -> Result<Config, String> {
        match serde_json::from_str::<Manifest>(&manifest_file_content) {
        Ok(manifest) => Ok(Config { root_path, patterns: manifest.workspaces, registry: String::from(registry) }),
        Err(_) => Err(String::from(
          "Fail to parse manifest file, please make sure it is a valid JSON and 'workspaces' array exists",
        ))
      }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jm_test_utils::*;

    #[test]
    fn fails_on_invalid_manifest_content() {
        let root_path = PathBuf::new();
        let content = "{}";
        let registry = "http://some/url";

        let result = Config::new(root_path, content, registry);

        assert_eq!(result, Err("Fail to parse manifest file, please make sure it is a valid JSON and 'workspaces' array exists".to_string()))
    }

    #[test]
    fn succeeds_on_valid_manifest_file() {
        let root_path = PathBuf::new();
        let content = get_manifest_file_content(vec!["packages/**", "not-in-packages/foo"]);
        let registry = String::from("http://some/url");

        let result = Config::new(root_path.clone(), &content, &registry);

        assert_eq!(
            result,
            Ok(Config {
                root_path,
                patterns: vec!["packages/**".to_string(), "not-in-packages/foo".to_string()],
                registry
            })
        )
    }
}
