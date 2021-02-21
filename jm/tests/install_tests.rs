use jm::cli_opts::{Command, Install, Opts};
use jm::run;
use jm_test_utils::*;

#[tokio::test]
async fn fails_on_missing_manifest_file() {
    given_manifest_file_does_not_exist(|path| async move {
        let opts = Opts {
            command: Command::Install(Install {}),
        };

        let result = run(path.to_path_buf(), opts).await;
        let expected = Err(String::from(
            "Couldn't find root directory. Make sure jm.json exists",
        ));

        assert_eq!(result, expected);
    })
    .await;
}

#[tokio::test]
async fn succeeds_when_manifest_file_is_valid() {
    given_valid_manifest_file(|path| async move {
        let opts = Opts {
            command: Command::Install(Install {}),
        };

        let result = run(path.to_path_buf(), opts).await;

        assert_eq!(result, Ok(()))
    })
    .await;
}
