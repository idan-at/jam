use async_trait::async_trait;
use jam_core::build_graph;
use jam_core::dependency::Dependency;
use jam_core::errors::JamCoreError;
use jam_core::package::NpmPackage;
use jam_core::package::Package;
use jam_core::package::WorkspacePackage;
use jam_core::resolver::PackageResolver;
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
    ) -> Result<(Package, &'a Dependency), JamCoreError> {
        Err(JamCoreError::new(String::from("Failing resolver")))
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
    ) -> Result<(Package, &'a Dependency), JamCoreError> {
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
        assert_eq!(err, JamCoreError::new(String::from("Failing resolver")))
    }
}

#[tokio::test]
async fn returns_monorepo_graph() {
    let base = vec![
        // TODO: test links when its the same major
        // TODO: test no links links when its the a different major
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
        Package::NpmPackage(NpmPackage::new(
            "dep1".to_string(),
            "1.0.0".to_string(),
            None,
            String::from("shasum"),
            String::from("tarball-url"),
        )),
    );
    resolver.given(
        Dependency {
            name: "dep2".to_string(),
            real_name: "dep2".to_string(),
            version_or_dist_tag: "1.0.0".to_string(),
        },
        Package::NpmPackage(NpmPackage::new(
            "dep2".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
              "dep3".to_string() => "~2.0.0".to_string()
            }),
            String::from("shasum"),
            String::from("tarball-url"),
        )),
    );
    resolver.given(
        Dependency {
            name: "dep3".to_string(),
            real_name: "dep3".to_string(),
            version_or_dist_tag: "~2.0.0".to_string(),
        },
        Package::NpmPackage(NpmPackage::new(
            "dep3".to_string(),
            "2.0.5".to_string(),
            None,
            String::from("shasum"),
            String::from("tarball-url"),
        )),
    );

    let result = build_graph(base, &resolver).await;

    assert!(result.is_ok());

    let (starting_nodes, graph) = result.unwrap();

    assert_eq!(starting_nodes.len(), 2);
    assert_eq!(graph.edge_count(), 4);
    assert_eq!(graph.node_count(), 5);
}

#[tokio::test]
async fn returns_monorepo_graph_when_it_has_cyclic_dependencies() {
    let base = vec![Package::WorkspacePackage(WorkspacePackage::new(
        "p1".to_string(),
        "1.0.0".to_string(),
        Some(hashmap! {
          "dep1".to_string() => "1.0.0".to_string()
        }),
        None,
        PathBuf::new(),
    ))];
    let mut resolver = MockResolver::new();

    resolver.given(
        Dependency {
            name: "dep1".to_string(),
            real_name: "dep1".to_string(),
            version_or_dist_tag: "1.0.0".to_string(),
        },
        Package::NpmPackage(NpmPackage::new(
            "dep1".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
                "dep2".to_string() => "1.0.0".to_string()
            }),
            String::from("shasum"),
            String::from("tarball-url"),
        )),
    );
    resolver.given(
        Dependency {
            name: "dep2".to_string(),
            real_name: "dep2".to_string(),
            version_or_dist_tag: "1.0.0".to_string(),
        },
        Package::NpmPackage(NpmPackage::new(
            "dep2".to_string(),
            "1.0.0".to_string(),
            Some(hashmap! {
              "dep1".to_string() => "1.0.0".to_string()
            }),
            String::from("shasum"),
            String::from("tarball-url"),
        )),
    );

    let result = build_graph(base, &resolver).await;

    assert!(result.is_ok());

    let (starting_nodes, graph) = result.unwrap();

    assert_eq!(starting_nodes.len(), 1);
    assert_eq!(graph.edge_count(), 3);
    assert_eq!(graph.node_count(), 3);
}
