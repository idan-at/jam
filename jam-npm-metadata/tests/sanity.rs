use jam_npm_metadata::*;
use maplit::hashmap;
use serde_json::from_str;

#[test]
fn test_basic_serialization_no_dependencies() {
    let metadata = r#"{
    "versions": {
      "1.0.0": {
        "dist": {
          "shasum": "shasum",
          "tarball": "tarball"
        }
      }
    }
  }"#;

    let result = from_str::<NpmPackageMetadata>(metadata).unwrap();
    let expected = NpmPackageMetadata {
        dist_tags: None,
        versions: hashmap! {
          "1.0.0".to_string() => NpmVersionMetadata {
            bin: None,
            dist: NpmDistMetadata {
              shasum: String::from("shasum"),
              tarball: String::from("tarball"),
            },
            dependencies: None
          }
        },
    };

    assert_eq!(result, expected);
}

#[test]
fn test_basic_serialization_with_dependencies() {
    let metadata = r#"{
    "versions": {
      "1.0.0": {
        "dist": {
          "shasum": "shasum",
          "tarball": "tarball"
        },
        "dependencies": {}
      }
    }
  }"#;

    let result = from_str::<NpmPackageMetadata>(metadata).unwrap();
    let expected = NpmPackageMetadata {
        dist_tags: None,
        versions: hashmap! {
          "1.0.0".to_string() => NpmVersionMetadata {
            bin: None,
            dist: NpmDistMetadata {
              shasum: String::from("shasum"),
              tarball: String::from("tarball"),
            },
            dependencies: Some(hashmap! {})
          }
        },
    };

    assert_eq!(result, expected);
}

#[test]
fn test_dist_tags_serialization() {
    let metadata = r#"{
    "dist-tags": {
      "latest": "1.0.0"
    },
    "versions": {
      "1.0.0": {
        "dist": {
          "shasum": "shasum",
          "tarball": "tarball"
        }
      }
    }
  }"#;

    let result = from_str::<NpmPackageMetadata>(metadata).unwrap();
    let expected = NpmPackageMetadata {
        dist_tags: Some(hashmap! {
          "latest".to_string() => "1.0.0".to_string()
        }),
        versions: hashmap! {
          "1.0.0".to_string() => NpmVersionMetadata {
            bin: None,
            dist: NpmDistMetadata {
              shasum: String::from("shasum"),
              tarball: String::from("tarball"),
            },
            dependencies: None
          }
        },
    };

    assert_eq!(result, expected);
}

#[test]
fn test_bin_serialization_as_string() {
    let metadata = r#"{
    "versions": {
      "1.0.0": {
        "bin": "./bin/script.js",
        "dist": {
          "shasum": "shasum",
          "tarball": "tarball"
        }
      }
    }
  }"#;

    let result = from_str::<NpmPackageMetadata>(metadata).unwrap();
    let expected = NpmPackageMetadata {
        dist_tags: None,
        versions: hashmap! {
          "1.0.0".to_string() => NpmVersionMetadata {
            bin: Some(NpmBinMetadata::String("./bin/script.js".to_string())),
            dist: NpmDistMetadata {
              shasum: String::from("shasum"),
              tarball: String::from("tarball"),
            },
            dependencies: None
          }
        },
    };

    assert_eq!(result, expected);
}

#[test]
fn test_bin_serialization_as_object() {
    let metadata = r#"{
    "versions": {
      "1.0.0": {
        "bin": {
          "name": "./bin/script.js"
        },
        "dist": {
          "shasum": "shasum",
          "tarball": "tarball"
        }
      }
    }
  }"#;

    let result = from_str::<NpmPackageMetadata>(metadata).unwrap();
    let expected = NpmPackageMetadata {
        dist_tags: None,
        versions: hashmap! {
          "1.0.0".to_string() => NpmVersionMetadata {
            bin: Some(NpmBinMetadata::Object(hashmap! { "name".to_string() => "./bin/script.js".to_string() })),
            dist: NpmDistMetadata {
              shasum: String::from("shasum"),
              tarball: String::from("tarball"),
            },
            dependencies: None
          }
        },
    };

    assert_eq!(result, expected);
}
