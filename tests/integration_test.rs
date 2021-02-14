mod common;

use common::*;

use jm::run;

#[test]
fn fails_on_missing_manifest_file() {
    let path = given_manifest_file_does_not_exist();

    let result = run(path.path().to_path_buf().clone());
    let expected = Err(format!(
        "Couldn't find manifest file in {:?}",
        path.path().join("package.json")
    ));

    assert_eq!(result, expected);
}

#[test]
fn succeeds_when_manifest_file_is_valid() {
    let path = given_valid_manifest_file();

    let result = run(path.path().to_path_buf().clone());

    assert_eq!(result, Ok(()))
}
