mod common;

use common::*;

use jm::run;

#[test]
fn fails_on_missing_manifest_file() {
  let path = given_manifest_file_does_not_exist();
  let path_buf = path.path().to_path_buf();

  let result = run(&path_buf);
  let expected = Err(format!("Couldn't find manifest file in {:?}", path_buf.join("package.json")));

  assert_eq!(result, expected);
}

#[test]
fn fails_on_invalid_manifest_file() {
  let path = given_invalid_manifest_file();
  let path_buf = path.path().to_path_buf();

  let result = run(&path_buf);
  let expected = Err(format!("Fail to parse {:?}, please make sure it is a valid JSON and 'workspaces' array exists", path_buf.join("package.json")));

  assert_eq!(result, expected);
}

#[test]
fn succeeds_when_manifest_file_is_valid() {
  let path = given_valid_manifest_file();

  let result = run(&path.path().to_path_buf());

  assert_eq!(result, Ok(()))
}
