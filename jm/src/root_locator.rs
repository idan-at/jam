use std::path::PathBuf;

const MANIFEST_FILE_NAME: &'static str = "jm.json";

// TODO: tests
pub fn find_root_dir(cwd: PathBuf) -> Result<PathBuf, String> {
    let possible_manifest_file_path = cwd.join(MANIFEST_FILE_NAME);

    if possible_manifest_file_path.exists() {
        Ok(cwd.clone())
    } else {
        match cwd.parent() {
            Some(parent) => find_root_dir(parent.to_path_buf()),
            None => Err(format!(
                "Couldn't find root directory. Make sure {} exists",
                MANIFEST_FILE_NAME
            )),
        }
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn finds_manifest_file_on_cwd() {
//         let root_path = PathBuf::new();
//         let content = "{}";

//         let result = Config::new(root_path, content);

//         assert_eq!(result, Err("Fail to parse manifest file, please make sure it is a valid JSON and 'workspaces' array exists".to_string()))
//     }

//     #[test]
//     fn succeeds_on_valid_manifest_file() {
//         let root_path = PathBuf::new();
//         let content = r#"{ "workspaces": [ "packages/**", "not-in-packages/foo" ]}"#;

//         let result = Config::new(root_path.clone(), content);

//         assert_eq!(
//             result,
//             Ok(Config {
//                 root_path,
//                 patterns: vec!["packages/**".to_string(), "not-in-packages/foo".to_string()]
//             })
//         )
//     }
// }
