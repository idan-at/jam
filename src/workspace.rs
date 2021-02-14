use crate::config::Config;

use globwalk::GlobWalkerBuilder;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Package {
    base_path: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct Workspace {
    pub packages: Vec<Package>,
}

impl Workspace {
    pub fn from_config(config: &Config) -> Result<Workspace, String> {
        let mut packages: Vec<Package> = Vec::new();

        let paths: Vec<String> = config
            .patterns
            .iter()
            .map(|pattern| String::from(format!("{}/package.json", pattern)))
            .collect();

        let walker = GlobWalkerBuilder::from_patterns(&config.root_path, &paths).build();

        match walker {
            Ok(walker) => {
                for entry in walker.into_iter().filter_map(Result::ok) {
                    if entry
                        .path()
                        .components()
                        .all(|component| component.as_os_str() != "node_modules")
                    {
                        packages.push(Package {
                            base_path: entry.path().parent().unwrap().to_path_buf(),
                        });
                    }
                }

                Ok(Workspace { packages })
            }
            Err(err) => Err(String::from(err.description())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    fn create_tmp_dir() -> TempDir {
        let tmp_dir = TempDir::new("jm_workspaces_fixtures").unwrap();

        tmp_dir
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
                packages: Vec::new()
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
        fs::write(p1_base_path.join("package.json"), "{}").unwrap();
        fs::write(p2_base_path.join("package.json"), "{}").unwrap();

        let config = Config::new(
            tmp_dir.path().to_path_buf().clone(),
            r#"{ "workspaces": ["**/*"] }"#,
        )
        .unwrap();

        let result = Workspace::from_config(&config);

        assert_eq!(
            result,
            Ok(Workspace {
                packages: vec![
                    Package {
                        base_path: p2_base_path
                    },
                    Package {
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
        fs::write(p1_base_path.join("package.json"), "{}").unwrap();
        fs::write(p2_base_path.join("package.json"), "{}").unwrap();

        let config = Config::new(
            tmp_dir.path().to_path_buf().clone(),
            r#"{ "workspaces": ["**/*", "!**/p2/**"] }"#,
        )
        .unwrap();

        let result = Workspace::from_config(&config);

        assert_eq!(
            result,
            Ok(Workspace {
                packages: vec![Package {
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
        fs::write(p1_base_path.join("package.json"), "{}").unwrap();
        fs::write(p2_base_path.join("package.json"), "{}").unwrap();

        let config = Config::new(
            tmp_dir.path().to_path_buf().clone(),
            r#"{ "workspaces": ["**/*"] }"#,
        )
        .unwrap();

        let result = Workspace::from_config(&config);

        assert_eq!(
            result,
            Ok(Workspace {
                packages: vec![Package {
                    base_path: p1_base_path
                }]
            })
        )
    }
}
