use std::collections::HashSet;
use crate::dependency::Dependency;
use crate::package::Package;

pub struct Collector {}

impl Collector {
    pub fn new() -> Collector {
        Collector {}
    }

    pub fn collect(&self, packages: &Vec<Package>) -> HashSet<Dependency> {
        packages
            .iter()
            .flat_map(|package| package.dependencies().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashset;

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
                dependencies: vec![dep1.clone()],
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

        let expected = hashset![dep1, dep2];

        assert_eq!(collector.collect(&packages), expected);
    }
}
