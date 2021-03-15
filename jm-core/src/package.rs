use crate::dependency::Dependency;
use log::warn;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<Dependency>,
    pub dev_dependencies: Vec<Dependency>,
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

    pub fn dependencies(&self) -> Vec<Dependency> {
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

fn to_dependencies_list(dependencies: Option<HashMap<String, String>>) -> Vec<Dependency> {
    let dependencies = dependencies.unwrap_or(HashMap::new());

    dependencies
        .iter()
        .map(|(key, value)| Dependency::from_entry(key, value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn collects_all_packages_dependencies_and_dev_dependencies() {
        let package = Package::new(
            String::from("some-package"),
            String::from("1.0.0"),
            Some(hashmap! {
                "lodash".to_string() => "1.0.0".to_string()
            }),
            Some(hashmap! {
                "lol".to_string() => "npm:lodash@~2.0.0".to_string()
            }),
        );

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
    fn uses_dependencies_over_dev_dependencies_in_case_of_repetitions() {
        let package = Package::new(
            String::from("some-package"),
            String::from("1.0.0"),
            Some(hashmap! {
                "lodash".to_string() => "1.0.0".to_string()
            }),
            Some(hashmap! {
                "lodash".to_string() => "~2.0.0".to_string()
            }),
        );

        let expected = vec![Dependency {
            name: "lodash".to_string(),
            real_name: "lodash".to_string(),
            version_or_dist_tag: "1.0.0".to_string(),
        }];

        assert_eq!(package.dependencies(), expected);
    }
}
