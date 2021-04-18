use futures::Future;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::common::*;

async fn with_tmp_dir<F>(func: impl FnOnce(PathBuf) -> F)
where
    F: Future<Output = ()>,
{
    let tmp_dir = create_tmp_dir();
    let path = tmp_dir.path().to_path_buf();

    func(path).await
}

pub async fn given_manifest_file_does_not_exist<F>(func: impl FnOnce(PathBuf) -> F)
where
    F: Future<Output = ()>,
{
    with_tmp_dir(func).await;
}

pub async fn given_valid_manifest_file<F>(func: impl FnOnce(PathBuf) -> F)
where
    F: Future<Output = ()>,
{
    with_tmp_dir(|path| async {
        let file_path = path.clone().join("jam.json");

        fs::write(file_path, with_manifest_file_content(vec!["**/*"])).unwrap();

        func(path).await;
    })
    .await;
}

pub async fn given_mono_repo_with<F>(
    contents: HashMap<PathBuf, String>,
    func: impl FnOnce(PathBuf) -> F,
) where
    F: Future<Output = ()>,
{
    given_valid_manifest_file(|path| async {
        for (package_relative_path, package_json_content) in contents {
            let package_path = path.join(package_relative_path);

            fs::create_dir_all(&package_path).unwrap();
            fs::write(package_path.join("package.json"), package_json_content).unwrap();
        }

        func(path).await;
    })
    .await
}
