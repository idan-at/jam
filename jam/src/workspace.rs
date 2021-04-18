use crate::common::read_manifest_file;
use crate::config::Config;
use crate::errors::JamError;
use jam_core::package::{Package, WorkspacePackage};

use globwalk::GlobWalkerBuilder;
use serde::Deserialize;
use std::collections::HashMap;

const IGNORE_PATTERS: [&str; 1] = ["!**/node_modules/**"];

#[derive(Debug, PartialEq, Deserialize)]
struct PackageJson {
    name: String,
    version: String,
    dependencies: Option<HashMap<String, String>>,
    #[serde(alias = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq)]
pub struct Workspace {
    pub workspace_packages: Vec<WorkspacePackage>,
}

impl Workspace {
    pub fn from_config(config: &Config) -> Result<Workspace, JamError> {
        let mut workspace_packages: Vec<WorkspacePackage> = Vec::new();

        let mut paths: Vec<String> = config
            .patterns
            .iter()
            .map(|pattern| format!("{}/package.json", pattern))
            .collect();

        paths.extend(
            IGNORE_PATTERS
                .iter()
                .map(|path| String::from(*path))
                .collect::<Vec<String>>(),
        );

        let walker = GlobWalkerBuilder::from_patterns(&config.root_path, &paths).build()?;

        for entry in walker.into_iter().filter_map(Result::ok) {
            let manifest_file_path = entry.path().to_path_buf();
            let manifest_file_content = read_manifest_file(manifest_file_path.clone())?;
            match serde_json::from_str::<PackageJson>(&manifest_file_content) {
                Ok(package_json) => workspace_packages.push(WorkspacePackage::new(
                    package_json.name,
                    package_json.version,
                    package_json.dependencies,
                    package_json.dev_dependencies,
                    entry.path().parent().unwrap().to_path_buf(),
                )),
                Err(_) => {
                    return Err(JamError::new(format!(
                        "Failed to parse {:?}",
                        manifest_file_path,
                    )))
                }
            }
        }

        if workspace_packages.len() == 0 {
            Err(JamError::new(format!("No packages were found in workspace")))
        } else {
            Ok(Workspace { workspace_packages })
        }
    }

    pub fn packages(&self) -> Vec<Package> {
        self.workspace_packages
            .iter()
            .map(|workspace_package| Package::WorkspacePackage(workspace_package.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jam_test_utils::common::*;
    use jam_test_utils::sync_helpers::*;
    use maplit::hashmap;
    use std::path::PathBuf;

    #[test]
    fn fails_on_invalid_package_json() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => String::from("{}")
        };

        given_mono_repo_with(contents, |path| {
            let registry = String::from("http://some/url");

            let config = Config::new(
                path.clone(),
                &with_manifest_file_content(vec!["**/*"]),
                &registry,
            )
            .unwrap();

            let result = Workspace::from_config(&config);

            assert_eq!(
                result,
                Err(JamError::new(format!(
                    "Failed to parse {:?}",
                    path.join("packages").join("p1").join("package.json"),
                )))
            )
        });
    }

    #[test]
    fn fails_when_no_package_matches_given_glob() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => String::from("{}")
        };

        given_mono_repo_with(contents, |path| {
            let registry = String::from("http://some/url");

            let config = Config::new(
                path.clone(),
                &with_manifest_file_content(vec!["packages/p2"]),
                &registry,
            )
            .unwrap();

            let result = Workspace::from_config(&config);

            assert_eq!(
                result,
                Err(JamError::new(String::from(
                    "No packages were found in workspace"
                )))
            )
        });
    }

    #[test]
    fn ignores_invalid_glob_pattern() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
        };

        given_mono_repo_with(contents, |path| {
            let registry = String::from("http://some/url");

            let config = Config::new(
                path.clone(),
                &with_manifest_file_content(vec!["?", "packages/p1"]),
                &registry,
            )
            .unwrap();

            let result = Workspace::from_config(&config);

            assert_eq!(
                result,
                Ok(Workspace {
                    workspace_packages: vec![WorkspacePackage {
                        base_path: path.join("packages").join("p1"),
                        name: String::from("p1"),
                        version: String::from("1.0.0"),
                        dependencies: vec![],
                        dev_dependencies: vec![],
                    }]
                })
            )
        });
    }

    #[test]
    fn collects_the_matching_manifest_files_parents() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
            PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.1.0", None)
        };

        given_mono_repo_with(contents, |path| {
            let registry = String::from("http://some/url");

            let config = Config::new(
                path.clone(),
                &with_manifest_file_content(vec!["**/*"]),
                &registry,
            )
            .unwrap();

            let result = Workspace::from_config(&config);

            assert_eq!(
                result,
                Ok(Workspace {
                    workspace_packages: vec![
                        WorkspacePackage {
                            base_path: path.join("packages").join("p2"),
                            name: String::from("p2"),
                            version: String::from("1.1.0"),
                            dependencies: vec![],
                            dev_dependencies: vec![],
                        },
                        WorkspacePackage {
                            base_path: path.join("packages").join("p1"),
                            name: String::from("p1"),
                            version: String::from("1.0.0"),
                            dependencies: vec![],
                            dev_dependencies: vec![],
                        }
                    ]
                })
            )
        });
    }

    #[test]
    fn takes_all_patterns_into_account() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
            PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.1.0", None)
        };

        given_mono_repo_with(contents, |path| {
            let registry = String::from("http://some/url");

            let config = Config::new(
                path.clone(),
                &with_manifest_file_content(vec!["**/*", "!**/p2/**"]),
                &registry,
            )
            .unwrap();

            let result = Workspace::from_config(&config);

            assert_eq!(
                result,
                Ok(Workspace {
                    workspace_packages: vec![WorkspacePackage {
                        base_path: path.join("packages").join("p1"),
                        name: String::from("p1"),
                        version: String::from("1.0.0"),
                        dependencies: vec![],
                        dev_dependencies: vec![],
                    }]
                })
            )
        });
    }

    #[test]
    fn ignores_packages_inside_node_modules() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0", None),
            PathBuf::from("packages/node_modules/p2") => with_package_json_file_content("p2", "1.1.0", None)
        };

        given_mono_repo_with(contents, |path| {
            let registry = String::from("http://some/url");

            let config = Config::new(
                path.clone(),
                &with_manifest_file_content(vec!["**/*", "!**/p2/**"]),
                &registry,
            )
            .unwrap();

            let result = Workspace::from_config(&config);

            assert_eq!(
                result,
                Ok(Workspace {
                    workspace_packages: vec![WorkspacePackage {
                        base_path: path.join("packages").join("p1"),
                        name: String::from("p1"),
                        version: String::from("1.0.0"),
                        dependencies: vec![],
                        dev_dependencies: vec![],
                    }]
                })
            )
        });
    }

    #[test]
    fn test_get_packages() {
        let workspace = Workspace {
            workspace_packages: vec![
                WorkspacePackage {
                    base_path: PathBuf::new(),
                    name: String::from("p1"),
                    version: String::from("1.0.0"),
                    dependencies: vec![],
                    dev_dependencies: vec![],
                },
                WorkspacePackage {
                    base_path: PathBuf::new(),
                    name: String::from("p2"),
                    version: String::from("2.0.0"),
                    dependencies: vec![],
                    dev_dependencies: vec![],
                },
            ],
        };

        let expected = vec![
            Package::WorkspacePackage(workspace.workspace_packages[0].clone()),
            Package::WorkspacePackage(workspace.workspace_packages[1].clone()),
        ];

        assert_eq!(workspace.packages(), expected);
    }
}
