use semver::{Compat, VersionReq};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Dependency {
    pub name: String,
    pub real_name: String,
    pub version_or_dist_tag: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<Dependency>,
    pub dev_dependencies: Vec<Dependency>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PackageNode {
    pub name: String,
    pub version: String,
}

impl Package {
    pub fn new(
        name: String,
        version: String,
        dependencies: Option<HashMap<String, String>>,
        dev_dependencies: Option<HashMap<String, String>>,
    ) -> Package {
        Package {
            name,
            version,
            dependencies: to_dependencies_list(dependencies),
            dev_dependencies: to_dependencies_list(dev_dependencies),
        }
    }

    // TODO: test
    // TODO: warning when package has a dependency that is also a dev dependency
    pub fn dependencies(&self) -> Vec<Dependency> {
        let dependencies = self.dependencies.clone();
        let dev_dependencies = self.dev_dependencies.clone();

        dependencies.into_iter().chain(dev_dependencies).collect()
    }
}

impl Dependency {
    // TODO: test
    pub fn from_entry(key: String, value: String) -> Dependency {
        match VersionReq::parse_compat(&value, Compat::Npm) {
            Ok(version) => Dependency {
                name: key.clone(),
                real_name: key,
                version_or_dist_tag: version.to_string(),
            },
            Err(_) => {
                if value.starts_with("npm:") {
                    let segments: Vec<&str> = value
                        .split("npm:")
                        .collect::<Vec<&str>>()
                        .get(1)
                        .unwrap()
                        .split("@")
                        .collect();

                    if segments.len() == 2 {
                        Dependency {
                            name: key,
                            real_name: segments[0].to_string(),
                            version_or_dist_tag: segments[1].to_string(),
                        }
                    } else {
                        Dependency {
                            name: key,
                            real_name: format!("@{}", segments[1]),
                            version_or_dist_tag: segments[2].to_string(),
                        }
                    }
                } else {
                    Dependency {
                        name: key.clone(),
                        real_name: key,
                        version_or_dist_tag: value,
                    }
                }
            }
        }
    }
}

fn to_dependencies_list(dependencies: Option<HashMap<String, String>>) -> Vec<Dependency> {
    let dependencies = dependencies.unwrap_or(HashMap::new());

    dependencies
        .iter()
        .map(|(key, value)| Dependency::from_entry(key.clone(), value.clone()))
        .collect()
}
