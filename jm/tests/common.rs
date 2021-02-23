use jm::npm::{NpmDistMetadata, NpmPackageMetadata, NpmVersionMetadata};
use maplit::hashmap;

pub fn with_npm_package_metadata(version: &str) -> NpmPackageMetadata {
    NpmPackageMetadata {
        dist_tags: None,
        versions: hashmap! {
          version.to_string() => NpmVersionMetadata {
            dist: NpmDistMetadata {
              shasum: String::from("some-shasum"),
              tarball: String::from("some-tarball"),
            },
            dependencies: None,
            dev_dependencies: None,
          },
        },
    }
}
