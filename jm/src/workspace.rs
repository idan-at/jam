use crate::common::read_manifest_file;
use crate::config::Config;

use globwalk::GlobWalkerBuilder;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const IGNORE_PATTERS: [&str; 1] = ["!**/node_modules/**"];

#[derive(Debug, PartialEq, Deserialize)]
pub struct Package {
    name: String,
    version: String,
    dependencies: Option<HashMap<String, String>>,
    #[serde(alias = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq)]
pub struct WorkspacePackage {
    base_path: PathBuf,
    package: Package,
}

#[derive(Debug, PartialEq)]
pub struct Workspace {
    pub workspace_packages: Vec<WorkspacePackage>,
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
                    match serde_json::from_str::<Package>(&manifest_file_content) {
                        Ok(package) => workspace_packages.push(WorkspacePackage {
                            base_path: entry.path().parent().unwrap().to_path_buf(),
                            package,
                        }),
                        Err(_) => return Err(format!("Fail to parse {:?}", manifest_file_path,)),
                    }
                }

                Ok(Workspace { workspace_packages })
            }
            Err(err) => Err(String::from(err.to_string())),
        }
    }

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
        dependencies: &Option<HashMap<String, String>>,
    ) {
        if let Some(dependencies) = dependencies {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_temp_dir;
    use std::fs;

    fn metadata_file_content(name: &str, version: &str) -> String {
        String::from(format!(
            r#"{{
            "name": "{}",
            "version": "{}"
        }}"#,
            name, version
        ))
    }

    #[test]
    fn fails_on_invalid_package_json() {
        let tmp_dir = create_tmp_dir();

        let p1_base_path = tmp_dir.path().join("packages").join("p1");

        fs::create_dir_all(&p1_base_path).unwrap();
        fs::write(p1_base_path.join("package.json"), "{}").unwrap();

        let config = Config::new(
            tmp_dir.path().to_path_buf().clone(),
            r#"{ "workspaces": ["**/*"] }"#,
        )
        .unwrap();

        let result = Workspace::from_config(&config);

        assert_eq!(
            result,
            Err(format!(
                "Fail to parse {:?}",
                p1_base_path.join("package.json"),
            ))
        )
    }

    #[test]
    fn ignores_invalid_glob_pattern() {
        let tmp_dir = create_tmp_dir();

        let config = Config::new(
            tmp_dir.path().to_path_buf().clone(),
            r#"{ "workspaces": ["?"] }"#,
        )
        .unwrap();

        let result = Workspace::from_config(&config);

        assert_eq!(
            result,
            Ok(Workspace {
                workspace_packages: Vec::new()
            })
        )
    }

    #[test]
    fn collects_the_matching_manifest_files_parents() {
        let tmp_dir = create_tmp_dir();

        let p1_base_path = tmp_dir.path().join("packages").join("p1");
        let p2_base_path = tmp_dir.path().join("packages").join("p2");

        fs::create_dir_all(&p1_base_path).unwrap();
        fs::create_dir_all(&p2_base_path).unwrap();
        fs::write(
            p1_base_path.join("package.json"),
            metadata_file_content("p1", "1.0.0"),
        )
        .unwrap();
        fs::write(
            p2_base_path.join("package.json"),
            metadata_file_content("p2", "1.1.0"),
        )
        .unwrap();

        let config = Config::new(
            tmp_dir.path().to_path_buf().clone(),
            r#"{ "workspaces": ["**/*"] }"#,
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
                            dependencies: None,
                            dev_dependencies: None
                        },
                        base_path: p2_base_path
                    },
                    WorkspacePackage {
                        package: Package {
                            name: String::from("p1"),
                            version: String::from("1.0.0"),
                            dependencies: None,
                            dev_dependencies: None
                        },
                        base_path: p1_base_path
                    }
                ]
            })
        )
    }

    #[test]
    fn takes_all_patterns_into_account() {
        let tmp_dir = create_tmp_dir();

        let p1_base_path = tmp_dir.path().join("packages").join("p1");
        let p2_base_path = tmp_dir.path().join("packages").join("p2");

        fs::create_dir_all(&p1_base_path).unwrap();
        fs::create_dir_all(&p2_base_path).unwrap();
        fs::write(
            p1_base_path.join("package.json"),
            metadata_file_content("p1", "1.0.0"),
        )
        .unwrap();
        fs::write(
            p2_base_path.join("package.json"),
            metadata_file_content("p2", "1.1.0"),
        )
        .unwrap();

        let config = Config::new(
            tmp_dir.path().to_path_buf().clone(),
            r#"{ "workspaces": ["**/*", "!**/p2/**"] }"#,
        )
        .unwrap();

        let result = Workspace::from_config(&config);

        assert_eq!(
            result,
            Ok(Workspace {
                workspace_packages: vec![WorkspacePackage {
                    package: Package {
                        name: String::from("p1"),
                        version: String::from("1.0.0"),
                        dependencies: None,
                        dev_dependencies: None
                    },
                    base_path: p1_base_path
                }]
            })
        )
    }

    #[test]
    fn ignores_packages_inside_node_modules() {
        let tmp_dir = create_tmp_dir();

        let p1_base_path = tmp_dir.path().join("packages").join("p1");
        let p2_base_path = tmp_dir
            .path()
            .join("packages")
            .join("node_modules")
            .join("p2");

        fs::create_dir_all(&p1_base_path).unwrap();
        fs::create_dir_all(&p2_base_path).unwrap();
        fs::write(
            p1_base_path.join("package.json"),
            metadata_file_content("p1", "1.0.0"),
        )
        .unwrap();
        fs::write(
            p2_base_path.join("package.json"),
            metadata_file_content("p2", "1.1.0"),
        )
        .unwrap();

        let config = Config::new(
            tmp_dir.path().to_path_buf().clone(),
            r#"{ "workspaces": ["**/*"] }"#,
        )
        .unwrap();

        let result = Workspace::from_config(&config);

        assert_eq!(
            result,
            Ok(Workspace {
                workspace_packages: vec![WorkspacePackage {
                    package: Package {
                        name: String::from("p1"),
                        version: String::from("1.0.0"),
                        dependencies: None,
                        dev_dependencies: None
                    },
                    base_path: p1_base_path
                }]
            })
        )
    }

    #[test]
    fn collects_all_dependencies_versions() {
        let workspace = Workspace {
            workspace_packages: vec![
                WorkspacePackage {
                    package: Package {
                        name: String::from("p1"),
                        version: String::from("1.0.0"),
                        dependencies: Some(
                            vec![
                                ("dep1".to_string(), "~1.0.0".to_string()),
                                ("dep2".to_string(), "~1.0.0".to_string()),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        dev_dependencies: Some(
                            vec![("dep3".to_string(), "1.0.0".to_string())]
                                .into_iter()
                                .collect(),
                        ),
                    },
                    base_path: PathBuf::from("/p1"),
                },
                WorkspacePackage {
                    package: Package {
                        name: String::from("p2"),
                        version: String::from("1.0.0"),
                        dependencies: Some(
                            vec![
                                ("dep3".to_string(), "1.0.0".to_string()),
                                ("dep2".to_string(), "~1.0.0".to_string()),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        dev_dependencies: Some(
                            vec![("dep1".to_string(), "~2.0.0".to_string())]
                                .into_iter()
                                .collect(),
                        ),
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
