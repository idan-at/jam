use std::path::PathBuf;

const MANIFEST_FILE_NAME: &'static str = "jm.json";

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

#[cfg(test)]
mod tests {
    use super::*;
    use jm_test_utils::*;
    use std::fs;

    #[test]
    fn fails_when_manifest_file_does_not_exist() {
        let tmp_dir = create_tmp_dir();

        let result = find_root_dir(tmp_dir.path().to_path_buf());

        assert_eq!(
            result,
            Err(String::from(
                "Couldn't find root directory. Make sure jm.json exists"
            ))
        );
    }

    #[test]
    fn finds_manifest_file_on_cwd() {
        let tmp_dir = create_tmp_dir();

        fs::write(tmp_dir.path().join("jm.json"), "{}").unwrap();

        let result = find_root_dir(tmp_dir.path().to_path_buf());

        assert_eq!(result, Ok(tmp_dir.path().to_path_buf()));
    }

    #[test]
    fn finds_manifest_file_on_parent() {
        let tmp_dir = create_tmp_dir();

        let sub_path = tmp_dir.path().join("sub1").join("sub2").join("sub3");

        fs::create_dir_all(&sub_path).unwrap();
        fs::write(tmp_dir.path().join("jm.json"), "{}").unwrap();

        let result = find_root_dir(sub_path.to_path_buf());

        assert_eq!(result, Ok(tmp_dir.path().to_path_buf()));
    }
}
