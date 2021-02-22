use crate::common::read_manifest_file;
use crate::config::Config;
use crate::package::Package;

use globwalk::GlobWalkerBuilder;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const IGNORE_PATTERS: [&str; 1] = ["!**/node_modules/**"];

#[derive(Debug, PartialEq, Deserialize)]
struct PackageJson {
    name: String,
    version: String,
    dependencies: Option<HashMap<String, String>>,
    #[serde(alias = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

impl PackageJson {
    pub fn to_package(self) -> Package {
        Package {
            name: self.name,
            version: self.version,
            dependencies: self.dependencies.unwrap_or(HashMap::new()),
            dev_dependencies: self.dev_dependencies.unwrap_or(HashMap::new()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct WorkspacePackage {
    base_path: PathBuf,
    package: Package,
}

#[derive(Debug, PartialEq)]
pub struct Workspace {
    workspace_packages: Vec<WorkspacePackage>,
}

impl Workspace {
    pub fn from_config(config: &Config) -> Result<Workspace, String> {
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

        let walker = GlobWalkerBuilder::from_patterns(&config.root_path, &paths).build();

        match walker {
            Ok(walker) => {
                for entry in walker.into_iter().filter_map(Result::ok) {
                    let manifest_file_path = entry.path().to_path_buf();
                    let manifest_file_content = read_manifest_file(manifest_file_path.clone())?;
                    match serde_json::from_str::<PackageJson>(&manifest_file_content) {
                        Ok(package_json) => workspace_packages.push(WorkspacePackage {
                            base_path: entry.path().parent().unwrap().to_path_buf(),
                            package: package_json.to_package(),
                        }),
                        Err(_) => return Err(format!("Fail to parse {:?}", manifest_file_path,)),
                    }
                }

                Ok(Workspace { workspace_packages })
            }
            Err(err) => Err(String::from(err.to_string())),
        }
    }

    // TODO: test
    pub fn packages(&self) -> Vec<&Package> {
        self.workspace_packages
            .iter()
            .map(|workspace_package| &workspace_package.package)
            .collect()
    }

    // TODO: remove
    pub fn collect_packages_versions(&self) -> HashMap<String, HashSet<String>> {
        self.workspace_packages
            .iter()
            .fold(HashMap::new(), |mut results, workspace_package| {
                self.append_packages_versions(
                    &mut results,
                    &workspace_package.package.dependencies,
                );
                self.append_packages_versions(
                    &mut results,
                    &workspace_package.package.dev_dependencies,
                );

                results
            })
    }

    fn append_packages_versions(
        &self,
        results: &mut HashMap<String, HashSet<String>>,
        dependencies: &HashMap<String, String>,
    ) {
        for (package_name, package_version) in dependencies {
            if let Some(versions) = results.get_mut(package_name) {
                versions.insert(package_version.to_string());
            } else {
                let versions_set: HashSet<String> =
                    vec![package_version.to_string()].iter().cloned().collect();

                results.insert(package_name.to_string(), versions_set);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jm_test_utils::common::*;
    use jm_test_utils::sync_helpers::*;
    use maplit::hashmap;

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
                Err(format!(
                    "Fail to parse {:?}",
                    path.join("packages").join("p1").join("package.json"),
                ))
            )
        });
    }

    #[test]
    fn ignores_invalid_glob_pattern() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => String::from("{}")
        };

        given_mono_repo_with(contents, |path| {
            let registry = String::from("http://some/url");

            let config = Config::new(
                path.clone(),
                &with_manifest_file_content(vec!["?"]),
                &registry,
            )
            .unwrap();

            let result = Workspace::from_config(&config);

            assert_eq!(
                result,
                Ok(Workspace {
                    workspace_packages: Vec::new()
                })
            )
        });
    }

    #[test]
    fn collects_the_matching_manifest_files_parents() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0"),
            PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.1.0")
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
                            package: Package {
                                name: String::from("p2"),
                                version: String::from("1.1.0"),
                                dependencies: HashMap::new(),
                                dev_dependencies: HashMap::new(),
                            },
                            base_path: path.join("packages").join("p2")
                        },
                        WorkspacePackage {
                            package: Package {
                                name: String::from("p1"),
                                version: String::from("1.0.0"),
                                dependencies: HashMap::new(),
                                dev_dependencies: HashMap::new(),
                            },
                            base_path: path.join("packages").join("p1")
                        }
                    ]
                })
            )
        });
    }

    #[test]
    fn takes_all_patterns_into_account() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0"),
            PathBuf::from("packages/p2") => with_package_json_file_content("p2", "1.1.0")
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
                    workspace_packages: vec![
                        WorkspacePackage {
                            package: Package {
                                name: String::from("p1"),
                                version: String::from("1.0.0"),
                                dependencies: HashMap::new(),
                                dev_dependencies: HashMap::new(),
                            },
                            base_path: path.join("packages").join("p1")
                        }
                    ]
                })
            )
        });
    }

    #[test]
    fn ignores_packages_inside_node_modules() {
        let contents = hashmap! {
            PathBuf::from("packages/p1") => with_package_json_file_content("p1", "1.0.0"),
            PathBuf::from("packages/node_modules/p2") => with_package_json_file_content("p2", "1.1.0")
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
                    workspace_packages: vec![
                        WorkspacePackage {
                            package: Package {
                                name: String::from("p1"),
                                version: String::from("1.0.0"),
                                dependencies: HashMap::new(),
                                dev_dependencies: HashMap::new(),
                            },
                            base_path: path.join("packages").join("p1")
                        }
                    ]
                })
            )
        });
    }

    #[test]
    fn collects_all_dependencies_versions() {
        let workspace = Workspace {
            workspace_packages: vec![
                WorkspacePackage {
                    package: Package {
                        name: String::from("p1"),
                        version: String::from("1.0.0"),
                        dependencies: vec![
                            ("dep1".to_string(), "~1.0.0".to_string()),
                            ("dep2".to_string(), "~1.0.0".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                        dev_dependencies: vec![("dep3".to_string(), "1.0.0".to_string())]
                            .into_iter()
                            .collect(),
                    },
                    base_path: PathBuf::from("/p1"),
                },
                WorkspacePackage {
                    package: Package {
                        name: String::from("p2"),
                        version: String::from("1.0.0"),
                        dependencies: vec![
                            ("dep3".to_string(), "1.0.0".to_string()),
                            ("dep2".to_string(), "~1.0.0".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                        dev_dependencies: vec![("dep1".to_string(), "~2.0.0".to_string())]
                            .into_iter()
                            .collect(),
                    },
                    base_path: PathBuf::from("/p2"),
                },
            ],
        };

        let packages_versions = workspace.collect_packages_versions();

        let expected: HashMap<String, HashSet<String>> = vec![
            (
                "dep3".to_string(),
                vec!["1.0.0".to_string()].iter().cloned().collect(),
            ),
            (
                "dep2".to_string(),
                vec!["~1.0.0".to_string()].iter().cloned().collect(),
            ),
            (
                "dep1".to_string(),
                vec!["~1.0.0".to_string(), "~2.0.0".to_string()]
                    .iter()
                    .cloned()
                    .collect(),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(packages_versions, expected);
    }
}
