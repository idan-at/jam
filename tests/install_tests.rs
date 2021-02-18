mod common;

use common::*;

use jm::run;
use jm::cli_opts::{Opts, Command, Install};

#[tokio::test]
async fn fails_on_missing_manifest_file() {
    let path = given_manifest_file_does_not_exist();
    let opts = Opts {
        command: Command::Install(Install {}),
    };

    let result = run(path.path().to_path_buf().clone(), opts).await;
    let expected = Err(format!(
        "Couldn't find manifest file in {:?}",
        path.path().join("jm.json")
    ));

    assert_eq!(result, expected);
}

#[tokio::test]
async fn succeeds_when_manifest_file_is_valid() {
    let path = given_valid_manifest_file();
    let opts = Opts {
        command: Command::Install(Install {}),
    };

    let result = run(path.path().to_path_buf().clone(), opts).await;

    assert_eq!(result, Ok(()))
}
