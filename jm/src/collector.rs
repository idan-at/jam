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
    let dep1 = Dependency {
      name: "dep1",
      real_name: "dep1",
      version_or_dist_tag: "latest"
    };
    let dep2 = Dependency {
      name: "dep1",
      real_name: "dep1",
      version_or_dist_tag: "latest"
    };

    let packages = vec![{
      Package {
        name: "p1".to_string(),
        version: "1.0.0".to_string(),
        dependencies: vec![dep1],
        dev_dependencies: vec![],
      },
      Package {
        name: "p2".to_string(),
        version: "1.0.0".to_string(),
        dependencies: vec![dep2],
        dev_dependencies: vec![],
      }
    }];

    let collector = Config::new(packages);

    let expected = vec![dep1, dep2];

    assert_eq!(collector.collect(), expected);
  }
}
