mod common;

use common::*;

use jm::install;

#[tokio::test]
async fn fails_on_missing_manifest_file() {
    let path = given_manifest_file_does_not_exist();

    let result = install(path.path().to_path_buf().clone()).await;
    let expected = Err(format!(
        "Couldn't find manifest file in {:?}",
        path.path().join("package.json")
    ));

    assert_eq!(result, expected);
}

#[tokio::test]
async fn succeeds_when_manifest_file_is_valid() {
    let path = given_valid_manifest_file();

    let result = install(path.path().to_path_buf().clone()).await;

    assert_eq!(result, Ok(()))
}
