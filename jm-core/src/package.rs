use crate::Dependency;
use crate::HashMap;
use log::warn;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NpmPackage {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<Dependency>,
    pub dev_dependencies: Vec<Dependency>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct WorkspacePackage {
    pub base_path: PathBuf,
    pub package: NpmPackage,
}

impl NpmPackage {
    pub fn new(
        name: String,
        version: String,
        dependencies: Option<HashMap<String, String>>,
        dev_dependencies: Option<HashMap<String, String>>,
    ) -> NpmPackage {
        NpmPackage {
            name,
            version,
            dependencies: to_dependencies_list(dependencies),
            dev_dependencies: to_dependencies_list(dev_dependencies),
        }
    }

    fn dependencies(&self) -> Vec<Dependency> {
        let dependencies = self.dependencies.clone();
        let dev_dependencies = self.dev_dependencies.clone();

        dependencies
            .into_iter()
            .chain(dev_dependencies)
            .fold(vec![], |mut acc, dependency| {
                if let Some(dependency) =
                    acc.iter().find(|existing| existing.name == dependency.name)
                {
                    warn!("Duplicate dependency {} in {}", dependency.name, self.name);
                } else {
                    acc.push(dependency);
                }

                acc
            })
    }
}

impl WorkspacePackage {
    pub fn new(
        name: String,
        version: String,
        dependencies: Option<HashMap<String, String>>,
        dev_dependencies: Option<HashMap<String, String>>,
        base_path: PathBuf,
    ) -> WorkspacePackage {
        let package = NpmPackage {
            name,
            version,
            dependencies: to_dependencies_list(dependencies),
            dev_dependencies: to_dependencies_list(dev_dependencies),
        };

        WorkspacePackage { package, base_path }
    }

    fn dependencies(&self) -> Vec<Dependency> {
        self.package.dependencies()
    }
}

fn to_dependencies_list(dependencies: Option<HashMap<String, String>>) -> Vec<Dependency> {
    let dependencies = dependencies.unwrap_or(HashMap::new());

    dependencies
        .iter()
        .map(|(key, value)| Dependency::from_entry(key, value))
        .collect()
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Package {
    Package(NpmPackage),
    WorkspacePackage(WorkspacePackage),
}

impl Package {
    pub fn name(&self) -> &str {
        match self {
            Package::Package(package) => &package.name,
            Package::WorkspacePackage(workspace_package) => &workspace_package.package.name,
        }
    }

    pub fn version(&self) -> &str {
        match self {
            Package::Package(package) => &package.version,
            Package::WorkspacePackage(workspace_package) => &workspace_package.package.version,
        }
    }

    pub fn dependencies(&self) -> Vec<Dependency> {
        match self {
            Package::Package(package) => package.dependencies(),
            Package::WorkspacePackage(workspace_package) => workspace_package.dependencies(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn exposes_name_and_version_getters() {
        let npm_package = Package::Package(NpmPackage::new(
            String::from("some-npm-package"),
            String::from("1.0.0"),
            None,
            None,
        ));

        let workspace_package = Package::WorkspacePackage(WorkspacePackage::new(
            String::from("some-workspace-package"),
            String::from("2.0.0"),
            None,
            None,
            PathBuf::new(),
        ));

        assert_eq!(npm_package.name(), "some-npm-package");
        assert_eq!(npm_package.version(), "1.0.0");
        assert_eq!(workspace_package.name(), "some-workspace-package");
        assert_eq!(workspace_package.version(), "2.0.0");
    }

    #[test]
    fn npm_package_collects_all_packages_dependencies_and_dev_dependencies() {
        let package = Package::Package(NpmPackage::new(
            String::from("some-package"),
            String::from("1.0.0"),
            Some(hashmap! {
                "lodash".to_string() => "1.0.0".to_string()
            }),
            Some(hashmap! {
                "lol".to_string() => "npm:lodash@~2.0.0".to_string()
            }),
        ));

        let expected = vec![
            Dependency {
                name: "lodash".to_string(),
                real_name: "lodash".to_string(),
                version_or_dist_tag: "1.0.0".to_string(),
            },
            Dependency {
                name: "lol".to_string(),
                real_name: "lodash".to_string(),
                version_or_dist_tag: "~2.0.0".to_string(),
            },
        ];

        assert_eq!(package.dependencies(), expected);
    }

    #[test]
    fn workspace_package_collects_all_packages_dependencies_and_dev_dependencies() {
        let package = Package::WorkspacePackage(WorkspacePackage::new(
            String::from("some-package"),
            String::from("1.0.0"),
            Some(hashmap! {
                "lodash".to_string() => "1.0.0".to_string()
            }),
            Some(hashmap! {
                "lol".to_string() => "npm:lodash@~2.0.0".to_string()
            }),
            PathBuf::new(),
        ));

        let expected = vec![
            Dependency {
                name: "lodash".to_string(),
                real_name: "lodash".to_string(),
                version_or_dist_tag: "1.0.0".to_string(),
            },
            Dependency {
                name: "lol".to_string(),
                real_name: "lodash".to_string(),
                version_or_dist_tag: "~2.0.0".to_string(),
            },
        ];

        assert_eq!(package.dependencies(), expected);
    }

    #[test]
    fn npm_package_uses_dependencies_over_dev_dependencies_in_case_of_repetitions() {
        let package = Package::Package(NpmPackage::new(
            String::from("some-package"),
            String::from("1.0.0"),
            Some(hashmap! {
                "lodash".to_string() => "1.0.0".to_string()
            }),
            Some(hashmap! {
                "lodash".to_string() => "~2.0.0".to_string()
            }),
        ));

        let expected = vec![Dependency {
            name: "lodash".to_string(),
            real_name: "lodash".to_string(),
            version_or_dist_tag: "1.0.0".to_string(),
        }];

        assert_eq!(package.dependencies(), expected);
    }

    #[test]
    fn workspace_package_uses_dependencies_over_dev_dependencies_in_case_of_repetitions() {
        let package = Package::WorkspacePackage(WorkspacePackage::new(
            String::from("some-package"),
            String::from("1.0.0"),
            Some(hashmap! {
                "lodash".to_string() => "1.0.0".to_string()
            }),
            Some(hashmap! {
                "lodash".to_string() => "~2.0.0".to_string()
            }),
            PathBuf::new(),
        ));

        let expected = vec![Dependency {
            name: "lodash".to_string(),
            real_name: "lodash".to_string(),
            version_or_dist_tag: "1.0.0".to_string(),
        }];

        assert_eq!(package.dependencies(), expected);
    }
}
