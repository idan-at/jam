use semver::{Compat, VersionReq};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Dependency {
    pub real_name: String,
    pub version_or_dist_tag: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, Dependency>,
    pub dev_dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PackageNode {
    pub name: String,
    pub version: String,
}

impl Package {
    // TODO: warning when package has a dependency that is also a dev dependency
    pub fn dependencies(&self) -> HashMap<String, Dependency> {
        let dependencies = self.dependencies.clone();
        let dev_dependencies = self.dev_dependencies.clone();

        dependencies.into_iter().chain(dev_dependencies).collect()
    }
}

impl Dependency {
    pub fn from_entry(key: String, value: String) -> Dependency {
        match VersionReq::parse_compat(&value, Compat::Npm) {
            Ok(version) => Dependency {
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
                            real_name: segments[0].to_string(),
                            version_or_dist_tag: segments[1].to_string(),
                        }
                    } else {
                        Dependency {
                            real_name: format!("@{}", segments[1]),
                            version_or_dist_tag: segments[2].to_string(),
                        }
                    }
                } else {
                    Dependency {
                        real_name: key,
                        version_or_dist_tag: value,
                    }
                }
            }
        }
    }
}

pub fn to_dependencies_hash_map(
    dependencies: Option<HashMap<String, String>>,
) -> HashMap<String, Dependency> {
    let dependencies = dependencies.unwrap_or(HashMap::new());

    dependencies
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                Dependency::from_entry(key.clone(), value.clone()),
            )
        })
        .collect()
}
