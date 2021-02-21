use futures::Future;
use std::fs;
use std::path::PathBuf;

use crate::sync_helpers::*;

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
        let file_path = path.clone().join("jm.json");

        fs::write(file_path, get_manifest_file_content(vec!["**/*"])).unwrap();

        func(path).await;
    })
    .await;
}
