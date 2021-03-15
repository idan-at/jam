use crate::dependency::Dependency;
use crate::package::Package;
use std::collections::HashMap;

pub struct Collector {}

impl Collector {
    pub fn new() -> Collector {
        Collector {}
    }

    pub fn collect(&self, packages: &Vec<Package>) -> HashMap<Dependency, Vec<Package>> {
        packages.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Dependency, Vec<Package>>, package| {
                for dependency in package.dependencies() {
                    match acc.get_mut(&dependency) {
                        Some(packages) => packages.push(package.clone()),
                        None => {
                            acc.insert(dependency.clone(), vec![package.clone()]);
                        }
                    };
                }

                acc
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn collects_all_packages_dependencies() {
        let dep1 = Dependency {
            name: "dep1".to_string(),
            real_name: "dep1".to_string(),
            version_or_dist_tag: "latest".to_string(),
        };
        let dep2 = Dependency {
            name: "dep1".to_string(),
            real_name: "dep1".to_string(),
            version_or_dist_tag: "latest".to_string(),
        };

        let packages = vec![
            Package {
                name: "p1".to_string(),
                version: "1.0.0".to_string(),
                dependencies: vec![dep1.clone(), dep2.clone()],
                dev_dependencies: vec![],
            },
            Package {
                name: "p2".to_string(),
                version: "1.0.0".to_string(),
                dependencies: vec![dep2.clone()],
                dev_dependencies: vec![],
            },
        ];

        let collector = Collector::new();

        let expected = hashmap! {
            dep1 => vec![packages[0].clone()],
            dep2 => vec![packages[0].clone(), packages[1].clone()],
        };

        assert_eq!(collector.collect(&packages), expected);
    }
}