use async_trait::async_trait;
use jm_core::build_graph;
use jm_core::dependency::Dependency;
use jm_core::package::Package;
use jm_core::package::WorkspacePackage;
use jm_core::resolver::PackageResolver;
use maplit::hashmap;
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

// #[tokio::test]
// async fn returns_monorepo_graph() {
//     let graph = build_graph().await;
// }
