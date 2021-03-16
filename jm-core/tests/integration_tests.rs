use async_trait::async_trait;
use jm_core::build_graph;
use jm_core::dependency::Dependency;
use jm_core::package::NpmPackage;
use jm_core::package::Package;
use jm_core::package::WorkspacePackage;
use jm_core::resolver::PackageResolver;
use maplit::hashmap;
use std::collections::HashMap;
use std::path::PathBuf;

struct FailingResolver {}

#[async_trait]
impl PackageResolver for FailingResolver {
    async fn get<'a>(
        &self,
        _requester: &str,
        _dependency: &'a Dependency,
    ) -> Result<(Package, &'a Dependency), String> {
        Err(String::from("Failing resolver"))
    }
}

struct MockResolver {
    store: HashMap<Dependency, Package>,
}

impl MockResolver {
    pub fn new() -> MockResolver {
        MockResolver {
            store: HashMap::new(),
        }
    }

    pub fn given(&mut self, dependency: Dependency, package: Package) {
        self.store.insert(dependency, package);
    }
}

#[async_trait]
impl PackageResolver for MockResolver {
    async fn get<'a>(
        &self,
        _requester: &str,
        dependency: &'a Dependency,
    ) -> Result<(Package, &'a Dependency), String> {
        Ok((self.store.get(dependency).unwrap().clone(), dependency))
    }
}

#[tokio::test]
async fn fails_when_resolver_fails() {
    let base = vec![Package::WorkspacePackage(WorkspacePackage::new(
        "name".to_string(),
        "1.0.0".to_string(),
        Some(hashmap! {
          "dep1".to_string() => "1.0.0".to_string()
        }),
        None,
        PathBuf::new(),
    ))];
    let resolver = FailingResolver {};

    let result = build_graph(base, &resolver).await;

    assert!(result.is_err());

    if let Err(err) = result {
        assert_eq!(err, String::from("Failing resolver"))
    }
}

#[tokio::test]
async fn returns_monorepo_graph() {
    let base = vec![
        Package::WorkspacePackage(WorkspacePackage::new(
            "p1".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
              "dep1".to_string() => "1.0.0".to_string()
            }),
            Some(hashmap! {
              "dep2".to_string() => "1.0.0".to_string()
            }),
            PathBuf::new(),
        )),
        Package::WorkspacePackage(WorkspacePackage::new(
            "p2".to_string(),
            "1.0.0".to_string(),
            None,
            Some(hashmap! {
              "dep2".to_string() => "1.0.0".to_string()
            }),
            PathBuf::new(),
        )),
    ];
    let mut resolver = MockResolver::new();

    resolver.given(
        Dependency {
            name: "dep1".to_string(),
            real_name: "dep1".to_string(),
            version_or_dist_tag: "1.0.0".to_string(),
        },
        Package::Package(NpmPackage::new(
            "dep1".to_string(),
            "1.0.0".to_string(),
            None,
            None,
        )),
    );
    resolver.given(
        Dependency {
            name: "dep2".to_string(),
            real_name: "dep2".to_string(),
            version_or_dist_tag: "1.0.0".to_string(),
        },
        Package::Package(NpmPackage::new(
            "dep2".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
              "dep3".to_string() => "~2.0.0".to_string()
            }),
            None,
        )),
    );
    resolver.given(
        Dependency {
            name: "dep3".to_string(),
            real_name: "dep3".to_string(),
            version_or_dist_tag: "~2.0.0".to_string(),
        },
        Package::Package(NpmPackage::new(
            "dep3".to_string(),
            "2.0.5".to_string(),
            None,
            None,
        )),
    );

    let graph = build_graph(base, &resolver).await;

    assert!(graph.is_ok());

    let graph = graph.unwrap();

    assert_eq!(graph.edge_count(), 4);
    assert_eq!(graph.node_count(), 5);
}