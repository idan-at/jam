use jm::npm::{NpmDistMetadata, NpmPackageMetadata, NpmVersionMetadata};
use maplit::hashmap;
use std::collections::HashMap;

pub fn with_npm_package_metadata(
    version: &str,
    dist_tags: Option<HashMap<String, String>>,
) -> NpmPackageMetadata {
    NpmPackageMetadata {
        dist_tags,
        versions: hashmap! {
          version.to_string() => NpmVersionMetadata {
            dist: NpmDistMetadata {
              shasum: String::from("some-shasum"),
              tarball: String::from("some-tarball"),
            },
            dependencies: None,
          },
        },
    }
}
