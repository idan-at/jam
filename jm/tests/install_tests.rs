mod common;
mod npm_mock_server;

use common::*;
use jm::cli_opts::{Command, Install, Opts};
use jm::run;
use jm_test_utils::async_helpers::*;
use jm_test_utils::common::*;
use maplit::hashmap;
use std::path::PathBuf;

use npm_mock_server::NpmMockServer;

fn setup() -> NpmMockServer {
    let npm_mock_server = NpmMockServer::new();

    npm_mock_server
}

#[tokio::test]
async fn fails_on_missing_manifest_file() {
    given_manifest_file_does_not_exist(|path| async move {
        let opts = Opts {
            registry: String::from("http://some/url"),
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
async fn with_empty_mono_repo() {
    given_valid_manifest_file(|path| async move {
        let opts = Opts {
            registry: String::from("http://some/url"),
            command: Command::Install(Install {}),
        };

        let result = run(path.to_path_buf(), opts).await;

        assert_eq!(result, Ok(()))
    })
    .await;
}

#[tokio::test]
async fn with_mono_repo_without_dependencies() {
    let npm_mock_server = setup();
    let contents = hashmap! {
        PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
        PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.0.0", None),
    };

    given_mono_repo_with(contents, |path| async move {
        let opts = Opts {
            registry: String::from(npm_mock_server.url()),
            command: Command::Install(Install {}),
        };

        let result = run(path.to_path_buf(), opts).await;

        assert_eq!(result, Ok(()))
    })
    .await;
}

#[tokio::test]
async fn with_mono_repo_with_version_dependencies() {
    let mut npm_mock_server = setup();
    let contents = hashmap! {
        PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
        PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.0.0", Some(hashmap! {
            "lodash" => "~4.17.0"
        })),
    };

    let metadata = with_npm_package_metadata("4.17.21", None);
    npm_mock_server.with_metadata("lodash", &metadata);

    given_mono_repo_with(contents, |path| async move {
        let opts = Opts {
            registry: String::from(npm_mock_server.url()),
            command: Command::Install(Install {}),
        };

        let result = run(path.to_path_buf(), opts).await;

        assert_eq!(result, Ok(()))
    })
    .await;
}

#[tokio::test]
async fn with_mono_repo_with_dist_tag_dependencies() {
    let mut npm_mock_server = setup();
    let contents = hashmap! {
        PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
        PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.0.0", Some(hashmap! {
            "lodash" => "latest"
        })),
    };

    let metadata = with_npm_package_metadata(
        "4.17.21",
        Some(hashmap! {
            "latest".to_string() => "4.17.21".to_string()
        }),
    );
    npm_mock_server.with_metadata("lodash", &metadata);

    given_mono_repo_with(contents, |path| async move {
        let opts = Opts {
            registry: String::from(npm_mock_server.url()),
            command: Command::Install(Install {}),
        };

        let result = run(path.to_path_buf(), opts).await;

        assert_eq!(result, Ok(()))
    })
    .await;
}

#[tokio::test]
async fn with_mono_repo_with_alias_dependencies() {
    let mut npm_mock_server = setup();
    let contents = hashmap! {
        PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
        PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.0.0", Some(hashmap! {
            "lol" => "npm:lodash@~4.17.0"
        })),
    };

    let metadata = with_npm_package_metadata("4.17.21", None);
    npm_mock_server.with_metadata("lodash", &metadata);

    given_mono_repo_with(contents, |path| async move {
        let opts = Opts {
            registry: String::from(npm_mock_server.url()),
            command: Command::Install(Install {}),
        };

        let result = run(path.to_path_buf(), opts).await;

        assert_eq!(result, Ok(()))
    })
    .await;
}
