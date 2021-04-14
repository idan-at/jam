mod common;

use common::*;
use jm::cli_options::{CliOptions, Command, Install};
use jm::errors::JmError;
use jm::run;
use jm_test_utils::async_helpers::*;
use jm_test_utils::common::*;
use jm_test_utils::npm_mock_server::*;
use maplit::hashmap;
use std::path::PathBuf;

fn setup() -> NpmMockServer {
    let npm_mock_server = NpmMockServer::new();

    npm_mock_server
}

#[tokio::test]
async fn fails_on_missing_manifest_file() {
    given_manifest_file_does_not_exist(|path| async move {
        let options = CliOptions {
            cache_group: String::from("tests"),
            registry: String::from("http://some/url"),
            command: Command::Install(Install {}),
            debug: false,
        };

        let result = run(path.to_path_buf(), options).await;
        let expected = Err(JmError::new(String::from(
            "Couldn't find root directory. Make sure jm.json exists",
        )));

        assert_eq!(result, expected);
    })
    .await;
}

#[tokio::test]
async fn with_empty_mono_repo() {
    given_valid_manifest_file(|path| async move {
        let options = CliOptions {
            cache_group: String::from("tests"),
            registry: String::from("http://some/url"),
            command: Command::Install(Install {}),
            debug: false,
        };

        let result = run(path.to_path_buf(), options).await;

        assert_eq!(
            result,
            Err(JmError::new(String::from(
                "No packages were found in workspace"
            )))
        )
    })
    .await;
}

#[tokio::test]
async fn with_simple_mono_repo() {
    let mut npm_mock_server = setup();
    let contents = hashmap! {
        PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
        PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.0.0", Some(hashmap! {
            "lib" => "~1.0.0",
            "p1" => "~1.0.0",
        })),
    };

    let lib_metadata = with_npm_package_metadata(
        "1.0.4",
        Some(hashmap! {
            "@types/lodash".to_string() => "~4.17.0".to_string()
        }),
        None,
        format!("{}/tarball/{}", npm_mock_server.url(), "lib"),
    );
    let types_lodash_metadata = with_npm_package_metadata(
        "4.17.21",
        None,
        None,
        format!("{}/tarball/{}", npm_mock_server.url(), "lodash"),
    );

    npm_mock_server.with_metadata("lib", &lib_metadata);
    npm_mock_server.with_tarball_data(
        "lib",
        hashmap! { "file.js".to_string() => "const x = 1;".to_string() },
    );

    npm_mock_server.with_metadata("@types/lodash", &types_lodash_metadata);
    npm_mock_server.with_tarball_data(
        "@types/lodash",
        hashmap! { "index.d.ts".to_string() => "declare const x = 2".to_string() },
    );

    given_mono_repo_with(contents, |path| async move {
        let options = CliOptions {
            cache_group: String::from("tests"),
            registry: String::from(npm_mock_server.url()),
            command: Command::Install(Install {}),
            debug: false,
        };

        let result = run(path.to_path_buf(), options).await;

        assert_eq!(result, Ok(()))
    })
    .await;
}

#[tokio::test]
async fn with_mono_repo_with_cyclic_dependencies() {
    let mut npm_mock_server = setup();
    let contents = hashmap! {
        PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
        PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.0.0", Some(hashmap! {
            "lib" => "~1.0.0",
            "p1" => "~1.0.0",
        })),
    };

    let lib_metadata = with_npm_package_metadata(
        "1.0.4",
        Some(hashmap! {
            "lodash".to_string() => "~4.17.0".to_string()
        }),
        None,
        format!("{}/tarball/{}", npm_mock_server.url(), "lib"),
    );
    let lodash_metadata = with_npm_package_metadata(
        "4.17.21",
        Some(hashmap! {
            "lib".to_string() => "^1.0.0".to_string()
        }),
        None,
        format!("{}/tarball/{}", npm_mock_server.url(), "lodash"),
    );

    npm_mock_server.with_metadata("lib", &lib_metadata);
    npm_mock_server.with_tarball_data(
        "lib",
        hashmap! { "file.js".to_string() => "const x = 1;".to_string() },
    );

    npm_mock_server.with_metadata("lodash", &lodash_metadata);
    npm_mock_server.with_tarball_data(
        "lodash",
        hashmap! { "file.js".to_string() => "const x = 2;".to_string() },
    );

    given_mono_repo_with(contents, |path| async move {
        let options = CliOptions {
            cache_group: String::from("tests"),
            registry: String::from(npm_mock_server.url()),
            command: Command::Install(Install {}),
            debug: false,
        };

        let result = run(path.to_path_buf(), options).await;

        assert_eq!(result, Ok(()))
    })
    .await;
}
