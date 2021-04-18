use jam_npm_metadata::*;
use maplit::hashmap;
use std::collections::HashMap;

pub fn with_npm_package_metadata(
    version: &str,
    dependencies: Option<HashMap<String, String>>,
    dist_tags: Option<HashMap<String, String>>,
    tarball_url: String,
) -> NpmPackageMetadata {
    NpmPackageMetadata {
        dist_tags,
        versions: hashmap! {
          version.to_string() => NpmVersionMetadata {
            bin: None,
            dist: NpmDistMetadata {
              shasum: String::from("some-shasum"),
              tarball: tarball_url,
            },
            dependencies,
          },
        },
    }
}
