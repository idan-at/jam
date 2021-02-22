use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::common::*;

fn with_tmp_dir(func: impl FnOnce(PathBuf) -> ()) {
    let tmp_dir = create_tmp_dir();
    let path = tmp_dir.path().to_path_buf();

    func(path)
}

pub fn given_manifest_file_does_not_exist(func: impl FnOnce(PathBuf) -> ()) {
    with_tmp_dir(func)
}

pub fn given_valid_manifest_file(func: impl FnOnce(PathBuf) -> ()) {
    with_tmp_dir(|path| {
        let file_path = path.clone().join("jm.json");

        fs::write(file_path, with_manifest_file_content(vec!["**/*"])).unwrap();

        func(path);
    })
}

pub fn given_mono_repo_with(contents: HashMap<PathBuf, String>, func: impl FnOnce(PathBuf) -> ()) {
    given_valid_manifest_file(|path| {
        for (package_relative_path, package_json_content) in contents {
            let package_path = path.join(package_relative_path);

            fs::create_dir_all(&package_path).unwrap();
            fs::write(
                package_path.join("package.json"),
                package_json_content,
            )
            .unwrap();
        }

        func(path);
    })
}
