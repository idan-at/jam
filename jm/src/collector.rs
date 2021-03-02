pub struct Collector {
  packages: &Vec<Packages>
}

impl Collector {
  pub fn new(packages: &Vec<Packages>) -> Collector {
    Collector { packages }
  }

  pub fn collect(&self) -> Vec<Dependency> {
    self.packages.iter().flat_map(|package| package.dependencies().clone() ))
  }
}

#[cfg(test)]
mod tests {

  #[test]
  fn collects_all_packages_dependencies() {
    let packages = vec![{
      Package {
        name: "p1".to_string(),
        version: "1.0.0".to_string(),
        dependencies: vec![Dependency {
          name: "dep1",
          real_name: "dep1",
          version_or_dist_tag: "latest"
        }],
        dev_dependencies: vec![],
      },
      Package {
        name: "p2".to_string(),
        version: "1.0.0".to_string(),
        dependencies: vec![Dependency {
          name: "dep1",
          real_name: "dep1",
          version_or_dist_tag: "latest"
        }],
        dev_dependencies: vec![],
      }
    }]
  }
}
